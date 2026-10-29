use crate::reporting_period_service::ReportingPeriodSqlRow;
use async_trait::async_trait;
use chrono::NaiveDate;
use sdq_model::{
    Assessor, Category, DemographicField, DemographicFilter, Posture, Progress, ReportingPeriod,
    SdqError, SdqProgressSummary, SdqScore, SdqSubmission, SdqSubmissionSummary, Statement,
};
use sdq_service::sdq::SdqService;
use serde_json::Value;
use sqlx::PgPool;
use std::{collections::HashMap, str::FromStr};
use uuid::Uuid;

pub struct SdqServiceSqlxImpl {
    pub pool: PgPool,
}

impl SdqServiceSqlxImpl {
    pub fn new(pool: PgPool) -> SdqServiceSqlxImpl {
        SdqServiceSqlxImpl { pool }
    }
}
#[derive(sqlx::FromRow)]
struct SdqSubmissionRow {
    score: i64,
    order: i64,
    statement_key: String,
    category: String,
    posture: String,
    description: Option<String>,
    is_true_positive: bool,
}

impl From<SdqSubmissionRow> for SdqScore {
    fn from(raw: SdqSubmissionRow) -> Self {
        SdqScore {
            score: raw.score,
            statement: Statement {
                category: Category {
                    category: raw.category,
                    posture: Posture::from_str(&raw.posture).unwrap_or(Posture::Unknown),
                },
                description: raw.description,
                is_true_positive: raw.is_true_positive,
                key: raw.statement_key,
                order: raw.order,
            },
        }
    }
}

#[async_trait]
impl SdqService for SdqServiceSqlxImpl {
    async fn record_response(&self, sdq: SdqSubmission) -> Result<(), SdqError> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|e| SdqError::Db(e.to_string()))?;
        for score in sdq.scores {
            sqlx::query(
                r#"
                INSERT INTO sdq (period_id, assessor, statement, score)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(sdq.period_id)
            .bind(sdq.assessor.to_string())
            .bind(score.statement.key)
            .bind(score.score)
            .execute(&mut *transaction)
            .await
            .map_err(|e| SdqError::Db(e.to_string()))?;
        }
        transaction
            .commit()
            .await
            .map_err(|e| SdqError::Db(e.to_string()))
    }
    async fn get_submission(
        &self,
        period_id: &Uuid,
        assessor: &Assessor,
    ) -> Result<SdqSubmission, SdqError> {
        sqlx::query_as::<_, SdqSubmissionRow>(
            r#"
            SELECT
            s.statement as statement_key,
            s.score::BIGINT as score,
            st.order::BIGINT as order,
            st.category as category,
            c.posture as posture,
            st.description as description,
            st.is_true_positive as is_true_positive
          FROM
            sdq s
          INNER JOIN sdq_statement st
          ON s.statement = st.statement_key
          INNER JOIN sdq_category c
          ON st.category = c.category
          WHERE period_id = $1 AND assessor = $2
            "#,
        )
        .bind(period_id)
        .bind(assessor.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SdqError::Db(e.to_string()))
        .map(|r| r.into_iter().map(SdqScore::from).collect())
        .map(|scores| SdqSubmission {
            period_id: period_id.clone(),
            assessor: assessor.clone(),
            scores,
        })
    }
    async fn get_summary(
        &self,
        period_id: &Uuid,
        assessor: &Assessor,
    ) -> Result<SdqSubmissionSummary, SdqError> {
        let row =
            sqlx::query_as::<_, (Uuid, NaiveDate, Option<Value>, Option<Value>, Option<i64>)>(
                r#"
            SELECT p.client_id, v.period_date, v.category_subtotals,
                   v.posture_subtotals, v.total_difficulties
            FROM sdq_single_summary_view v
            JOIN reporting_period p ON p.period_id = v.period_id
            WHERE v.period_id = $1 AND v.assessor = $2
            "#,
            )
            .bind(period_id)
            .bind(assessor.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| SdqError::Db(e.to_string()))?
            .ok_or_else(|| SdqError::InvalidInput("No SDQ summary found".into()))?;

        let category_sub_totals = row
            .2
            .map(serde_json::from_value)
            .transpose()
            .map_err(|e| SdqError::InternalError(e.to_string()))?
            .unwrap_or_default();
        let posture_sub_totals = row
            .3
            .map(serde_json::from_value)
            .transpose()
            .map_err(|e| SdqError::InternalError(e.to_string()))?
            .unwrap_or_default();
        let total_difficulties = u64::try_from(row.4.unwrap_or_default())
            .map_err(|e| SdqError::InternalError(e.to_string()))?;

        Ok(SdqSubmissionSummary {
            client_id: row.0,
            assessor: assessor.clone(),
            period: row.1,
            category_sub_totals,
            posture_sub_totals,
            total_difficulties,
        })
    }
    async fn get_reporting_periods(
        &self,
        client_id: &Uuid,
    ) -> Result<Vec<ReportingPeriod>, SdqError> {
        sqlx::query_as::<_, ReportingPeriodSqlRow>(
            "SELECT period_id, client_id, period_date FROM reporting_period WHERE client_id = $1",
        )
        .bind(client_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SdqError::Db(e.to_string()))
        .map(|rows| rows.into_iter().map(ReportingPeriod::from).collect())
    }
    async fn query_sdq_progress(
        &self,
        assessors: &Vec<Assessor>,
        filters: &Vec<DemographicFilter>,
        from: &NaiveDate,
        to: &NaiveDate,
    ) -> Result<Vec<SdqProgressSummary>, SdqError> {
        let mut sql = String::from(
            r#"
            SELECT p.client_id, p.assessor, p.category_progress,
                   p.posture_progress, p.total_progress
            FROM sdq_progress_view p
            JOIN client c ON c.client_id = p.client_id
            WHERE p.assessor = ANY($1)
              AND p.last_period_date >= $2
              AND p.last_period_date < $3
            "#,
        );
        let mut parameters = Vec::new();
        for (index, filter) in filters.iter().enumerate() {
            let parameter = index + 4;
            let condition = match &filter.field {
                DemographicField::Gender => format!("c.gender = ANY(${parameter})"),
                DemographicField::Council => format!("c.council = ANY(${parameter})"),
                DemographicField::Ethnicity => format!("c.ethnicity = ANY(${parameter})"),
                DemographicField::EAL => format!("c.eal = ANY(${parameter})"),
                DemographicField::DisabilityStatus => {
                    format!("c.disability_status = ANY(${parameter})")
                }
                DemographicField::CareExperience => {
                    format!("c.care_experience = ANY(${parameter})")
                }
                DemographicField::FundingSource => {
                    format!("c.funding_source = ANY(${parameter})")
                }
                DemographicField::DisabilityType => format!(
                    "EXISTS (SELECT 1 FROM disability_type d WHERE d.client_id = p.client_id AND d.disability_type = ANY(${parameter}))"
                ),
                DemographicField::ACES => format!(
                    "EXISTS (SELECT 1 FROM aces a WHERE a.client_id = p.client_id AND a.ace_type = ANY(${parameter}))"
                ),
                DemographicField::InterventionType => format!(
                    "EXISTS (SELECT 1 FROM intervention_type i WHERE i.client_id = p.client_id AND i.intervention_type = ANY(${parameter}))"
                ),
            };
            sql.push_str(" AND ");
            sql.push_str(&condition);
            parameters.push(filter.values.clone());
        }

        let assessors = assessors
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        let mut query = sqlx::query_as::<
            _,
            (
                Uuid,
                Option<String>,
                Option<Value>,
                Option<Value>,
                Option<Value>,
            ),
        >(sqlx::AssertSqlSafe(sql.as_str()))
        .bind(assessors)
        .bind(from)
        .bind(to);
        for values in parameters {
            query = query.bind(values);
        }

        let rows = query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| SdqError::Db(e.to_string()))?;
        rows.into_iter()
            .map(|(client_id, assessor, categories, postures, total)| {
                let parse_progress = |value: Option<Value>| {
                    let values = value
                        .map(serde_json::from_value::<HashMap<String, HashMap<String, u64>>>)
                        .transpose()
                        .map_err(|e| SdqError::InternalError(e.to_string()))?
                        .unwrap_or_default();
                    values
                        .into_iter()
                        .map(|(key, progress)| {
                            let error_key = key.clone();
                            let get = |name: &str| {
                                progress.get(name).copied().ok_or_else(|| {
                                    SdqError::InternalError(format!(
                                        "Missing '{name}' value in progress for {error_key}"
                                    ))
                                })
                            };
                            Ok((
                                key,
                                Progress {
                                    first: get("first")?,
                                    last: get("last")?,
                                    delta: get("delta")?,
                                },
                            ))
                        })
                        .collect::<Result<HashMap<_, _>, SdqError>>()
                };
                Ok(SdqProgressSummary {
                    client_id,
                    assessor: assessor
                        .and_then(|value| Assessor::from_str(&value).ok())
                        .unwrap_or(Assessor::Unknown),
                    category_progress: parse_progress(categories)?,
                    posture_progress: parse_progress(postures)?,
                    total_difficulties: {
                        let progress = total
                            .map(serde_json::from_value::<HashMap<String, u64>>)
                            .transpose()
                            .map_err(|e| SdqError::InternalError(e.to_string()))?
                            .unwrap_or_default();
                        let get = |name: &str| {
                            progress.get(name).copied().ok_or_else(|| {
                                SdqError::InternalError(format!(
                                    "Missing '{name}' value in total progress"
                                ))
                            })
                        };
                        Progress {
                            first: get("first")?,
                            last: get("last")?,
                            delta: get("delta")?,
                        }
                    },
                })
            })
            .collect()
    }

    async fn query_sdq_summaries(
        &self,
        assessors: &Vec<Assessor>,
        filters: &Vec<DemographicFilter>,
        from: &NaiveDate,
        to: &NaiveDate,
    ) -> Result<HashMap<Uuid, HashMap<NaiveDate, Vec<SdqSubmissionSummary>>>, SdqError> {
        let mut sql = String::from(
            r#"
            SELECT v.client_id, v.period_date, v.assessor, v.category_subtotals,
                   v.posture_subtotals, v.total_difficulties
            FROM sdq_summary_view v
            JOIN client c ON c.client_id = v.client_id
            WHERE v.assessor = ANY($1)
              AND v.period_date >= $2
              AND v.period_date < $3
            "#,
        );
        let mut parameters = Vec::new();
        for (index, filter) in filters.iter().enumerate() {
            let parameter = index + 4;
            let condition = match &filter.field {
                DemographicField::Gender => format!("c.gender = ANY(${parameter})"),
                DemographicField::Council => format!("c.council = ANY(${parameter})"),
                DemographicField::Ethnicity => format!("c.ethnicity = ANY(${parameter})"),
                DemographicField::EAL => format!("c.eal = ANY(${parameter})"),
                DemographicField::DisabilityStatus => {
                    format!("c.disability_status = ANY(${parameter})")
                }
                DemographicField::CareExperience => {
                    format!("c.care_experience = ANY(${parameter})")
                }
                DemographicField::FundingSource => {
                    format!("c.funding_source = ANY(${parameter})")
                }
                DemographicField::DisabilityType => format!(
                    "EXISTS (SELECT 1 FROM disability_type d WHERE d.client_id = v.client_id AND d.disability_type = ANY(${parameter}))"
                ),
                DemographicField::ACES => format!(
                    "EXISTS (SELECT 1 FROM aces a WHERE a.client_id = v.client_id AND a.ace_type = ANY(${parameter}))"
                ),
                DemographicField::InterventionType => format!(
                    "EXISTS (SELECT 1 FROM intervention_type i WHERE i.client_id = v.client_id AND i.intervention_type = ANY(${parameter}))"
                ),
            };
            sql.push_str(" AND ");
            sql.push_str(&condition);
            parameters.push(filter.values.clone());
        }

        let assessors = assessors
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        let mut query = sqlx::query_as::<
            _,
            (
                Uuid,
                NaiveDate,
                Option<String>,
                Option<Value>,
                Option<Value>,
                Option<i64>,
            ),
        >(sqlx::AssertSqlSafe(sql.as_str()))
        .bind(assessors)
        .bind(from)
        .bind(to);
        for values in parameters {
            query = query.bind(values);
        }
        let rows = query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| SdqError::Db(e.to_string()))?;

        let mut summaries = HashMap::new();
        for (client_id, period, assessor, categories, postures, total) in rows {
            let category_sub_totals = categories
                .map(serde_json::from_value)
                .transpose()
                .map_err(|e| SdqError::InternalError(e.to_string()))?
                .unwrap_or_default();
            let posture_sub_totals = postures
                .map(serde_json::from_value)
                .transpose()
                .map_err(|e| SdqError::InternalError(e.to_string()))?
                .unwrap_or_default();
            let total_difficulties = u64::try_from(total.unwrap_or_default())
                .map_err(|e| SdqError::InternalError(e.to_string()))?;
            summaries
                .entry(client_id)
                .or_insert_with(HashMap::new)
                .entry(period)
                .or_insert_with(Vec::new)
                .push(SdqSubmissionSummary {
                    client_id,
                    assessor: assessor
                        .and_then(|value| Assessor::from_str(&value).ok())
                        .unwrap_or(Assessor::Unknown),
                    period,
                    category_sub_totals,
                    posture_sub_totals,
                    total_difficulties,
                });
        }
        Ok(summaries)
    }

    async fn get_sdq_progress_for_client(
        &self,
        client_id: &Uuid,
        assessor: &Assessor,
    ) -> Result<SdqProgressSummary, SdqError> {
        let row = sqlx::query_as::<_, (Option<Value>, Option<Value>, Option<Value>)>(
            r#"
            SELECT category_progress, posture_progress, total_progress
            FROM sdq_progress_view
            WHERE client_id = $1 AND assessor = $2
            "#,
        )
        .bind(client_id)
        .bind(assessor.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SdqError::Db(e.to_string()))?
        .ok_or_else(|| SdqError::InvalidInput("No progress found for client".into()))?;

        let parse_progress = |value: Option<Value>| {
            let values = value
                .map(serde_json::from_value::<HashMap<String, HashMap<String, u64>>>)
                .transpose()
                .map_err(|e| SdqError::InternalError(e.to_string()))?
                .unwrap_or_default();
            values
                .into_iter()
                .map(|(key, progress)| {
                    let error_key = key.clone();
                    let get = |name: &str| {
                        progress.get(name).copied().ok_or_else(|| {
                            SdqError::InternalError(format!(
                                "Missing '{name}' value in progress for {error_key}"
                            ))
                        })
                    };
                    Ok((
                        key,
                        Progress {
                            first: get("first")?,
                            last: get("last")?,
                            delta: get("delta")?,
                        },
                    ))
                })
                .collect::<Result<HashMap<_, _>, SdqError>>()
        };
        let total = row
            .2
            .map(serde_json::from_value::<HashMap<String, u64>>)
            .transpose()
            .map_err(|e| SdqError::InternalError(e.to_string()))?
            .unwrap_or_default();
        let total_value = |name: &str| {
            total.get(name).copied().ok_or_else(|| {
                SdqError::InternalError(format!("Missing '{name}' value in total progress"))
            })
        };

        Ok(SdqProgressSummary {
            client_id: *client_id,
            assessor: assessor.clone(),
            category_progress: parse_progress(row.0)?,
            posture_progress: parse_progress(row.1)?,
            total_difficulties: Progress {
                first: total_value("first")?,
                last: total_value("last")?,
                delta: total_value("delta")?,
            },
        })
    }
}
