use async_trait::async_trait;
use sdq_model::{
    Assessor, DemographicFilter, GboSubmission, Goal, GoalProgress, GoalType, SdqError,
};
use sdq_service::gbo::GboService;
use sdq_service::goal::GoalService;
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;

pub struct GoalServiceSqlxImpl {
    pub pool: PgPool,
}

impl GoalServiceSqlxImpl {
    pub fn new(pool: PgPool) -> GoalServiceSqlxImpl {
        GoalServiceSqlxImpl { pool }
    }
}

#[async_trait]
impl GoalService for GoalServiceSqlxImpl {
    async fn save_goal(&self, goal: Goal) -> Result<(), SdqError> {
        let client_id = goal
            .client_id
            .ok_or_else(|| SdqError::InvalidInput("Goal is missing clientId".into()))?;
        let goal_id = goal
            .goal_id
            .ok_or_else(|| SdqError::InvalidInput("Goal is missing goalId".into()))?;
        let default_type = GoalType::Unknown;
        let goal_type = serde_json::to_value(goal.r#type.as_ref().unwrap_or(&default_type))
            .map_err(|e| SdqError::InternalError(e.to_string()))?;

        sqlx::query(
            "INSERT INTO goal (client_id, goal_id, type, description) VALUES ($1, $2, $3, $4)",
        )
        .bind(client_id)
        .bind(goal_id)
        .bind(goal_type.as_str())
        .bind(goal.description)
        .execute(&self.pool)
        .await
        .map_err(|e| SdqError::Db(e.to_string()))
        .map(|_| ())
    }
    async fn get_for_client(&self, client_id: &Uuid) -> Result<Vec<Goal>, SdqError> {
        let rows = sqlx::query_as::<_, (Uuid, Option<String>, Option<String>)>(
            "SELECT goal_id, type, description FROM goal WHERE client_id = $1",
        )
        .bind(client_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SdqError::Db(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|(goal_id, goal_type, description)| Goal {
                client_id: Some(*client_id),
                goal_id: Some(goal_id),
                r#type: goal_type
                    .and_then(|value| GoalType::from_str(&value).ok())
                    .or(Some(GoalType::Unknown)),
                description,
            })
            .collect())
    }
    async fn delete_all_goals(&self) -> Result<(), SdqError> {
        sqlx::query("DELETE FROM goal")
            .execute(&self.pool)
            .await
            .map_err(|e| SdqError::Db(e.to_string()))
            .map(|_| ())
    }
    async fn get_goals_with_progress(
        &self,
        assessors: &Vec<Assessor>,
        filters: &Vec<DemographicFilter>,
        min_progress: u64,
        goal_types: &Vec<GoalType>,
        from: &chrono::NaiveDate,
        to: &chrono::NaiveDate,
    ) -> Result<Vec<GoalProgress>, SdqError> {
        if min_progress > i64::MAX as u64 {
            return Ok(Vec::new());
        }
        let mut sql = String::from(
            r#"
            SELECT s.client_id, s.goal_id, s.goal_type, s.goal_description,
                   s.assessor, s.first_score, s.last_score
            FROM goal_progress_view s
            JOIN client c ON c.client_id = s.client_id
            WHERE s.last_period_date >= $1 AND s.last_period_date < $2
              AND s.last_score::BIGINT - s.first_score::BIGINT >= $3
            "#,
        );
        let mut filter_values = Vec::new();
        let mut next_parameter = 4;

        if !assessors.is_empty() {
            sql.push_str(&format!(" AND s.assessor = ANY(${next_parameter})"));
            filter_values.push(
                assessors
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
            );
            next_parameter += 1;
        }
        if !goal_types.is_empty() {
            let values = goal_types
                .iter()
                .map(serde_json::to_value)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| SdqError::InternalError(e.to_string()))?
                .into_iter()
                .map(|value| {
                    value
                        .as_str()
                        .map(str::to_owned)
                        .ok_or_else(|| SdqError::InternalError("Invalid goal type".into()))
                })
                .collect::<Result<Vec<_>, _>>()?;
            sql.push_str(&format!(" AND s.goal_type = ANY(${next_parameter})"));
            filter_values.push(values);
            next_parameter += 1;
        }
        for filter in filters {
            let column = match filter.field {
                sdq_model::DemographicField::Gender => "c.gender",
                sdq_model::DemographicField::Council => "c.council",
                sdq_model::DemographicField::Ethnicity => "c.ethnicity",
                sdq_model::DemographicField::EAL => "c.eal",
                sdq_model::DemographicField::DisabilityStatus => "c.disability_status",
                sdq_model::DemographicField::CareExperience => "c.care_experience",
                sdq_model::DemographicField::FundingSource => "c.funding_source",
                sdq_model::DemographicField::DisabilityType => {
                    sql.push_str(&format!(
                        " AND EXISTS (SELECT 1 FROM disability_type d WHERE d.client_id = s.client_id AND d.disability_type = ANY(${next_parameter}))"
                    ));
                    filter_values.push(filter.values.clone());
                    next_parameter += 1;
                    continue;
                }
                sdq_model::DemographicField::ACES => {
                    sql.push_str(&format!(
                        " AND EXISTS (SELECT 1 FROM aces a WHERE a.client_id = s.client_id AND a.ace_type = ANY(${next_parameter}))"
                    ));
                    filter_values.push(filter.values.clone());
                    next_parameter += 1;
                    continue;
                }
                sdq_model::DemographicField::InterventionType => {
                    sql.push_str(&format!(
                        " AND EXISTS (SELECT 1 FROM intervention_type i WHERE i.client_id = s.client_id AND i.intervention_type = ANY(${next_parameter}))"
                    ));
                    filter_values.push(filter.values.clone());
                    next_parameter += 1;
                    continue;
                }
            };
            sql.push_str(&format!(" AND {column} = ANY(${next_parameter})"));
            filter_values.push(filter.values.clone());
            next_parameter += 1;
        }

        let mut query = sqlx::query_as::<
            _,
            (
                Uuid,
                Uuid,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<i32>,
                Option<i32>,
            ),
        >(sqlx::AssertSqlSafe(sql.as_str()))
        .bind(from)
        .bind(to)
        .bind(min_progress as i64);
        for values in filter_values {
            query = query.bind(values);
        }

        let rows = query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| SdqError::Db(e.to_string()))?;
        rows.into_iter()
            .map(
                |(
                    client_id,
                    goal_id,
                    goal_type,
                    description,
                    assessor,
                    first_score,
                    last_score,
                )| {
                    Ok(GoalProgress {
                        goal: Goal {
                            client_id: Some(client_id),
                            goal_id: Some(goal_id),
                            r#type: goal_type
                                .and_then(|value| GoalType::from_str(&value).ok())
                                .or(Some(GoalType::Unknown)),
                            description,
                        },
                        assessor: assessor
                            .and_then(|value| Assessor::from_str(&value).ok())
                            .unwrap_or(Assessor::Unknown),
                        first_score,
                        last_score,
                    })
                },
            )
            .collect()
    }
    async fn get_goals_with_progress_for_client(
        &self,
        client_id: &Uuid,
        assessor: Assessor,
    ) -> Result<Vec<GoalProgress>, SdqError> {
        let rows = sqlx::query_as::<
            _,
            (
                Uuid,
                Uuid,
                Option<String>,
                Option<String>,
                Option<i32>,
                Option<i32>,
            ),
        >(
            r#"
            SELECT client_id, goal_id, goal_type, goal_description, first_score, last_score
            FROM goal_progress_view
            WHERE client_id = $1 AND assessor = $2
            "#,
        )
        .bind(client_id)
        .bind(assessor.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SdqError::Db(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(
                |(client_id, goal_id, goal_type, description, first_score, last_score)| {
                    GoalProgress {
                        goal: Goal {
                            client_id: Some(client_id),
                            goal_id: Some(goal_id),
                            r#type: goal_type
                                .and_then(|value| GoalType::from_str(&value).ok())
                                .or(Some(GoalType::Unknown)),
                            description,
                        },
                        assessor: assessor.clone(),
                        first_score,
                        last_score,
                    }
                },
            )
            .collect())
    }
    async fn get_goal_progress(
        &self,
        goal_id: &Uuid,
        assessor: Assessor,
    ) -> Result<GoalProgress, SdqError> {
        let row = sqlx::query_as::<
            _,
            (
                Uuid,
                Uuid,
                Option<String>,
                Option<String>,
                Option<i32>,
                Option<i32>,
            ),
        >(
            r#"
            SELECT client_id, goal_id, goal_type, goal_description, first_score, last_score
            FROM goal_progress_view
            WHERE goal_id = $1 AND assessor = $2
            "#,
        )
        .bind(goal_id)
        .bind(assessor.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SdqError::Db(e.to_string()))?;

        if let Some((client_id, goal_id, goal_type, description, first_score, last_score)) = row {
            return Ok(GoalProgress {
                goal: Goal {
                    client_id: Some(client_id),
                    goal_id: Some(goal_id),
                    r#type: goal_type
                        .and_then(|value| GoalType::from_str(&value).ok())
                        .or(Some(GoalType::Unknown)),
                    description,
                },
                assessor,
                first_score,
                last_score,
            });
        }

        let goal = self.get_goal(goal_id).await?;
        Ok(GoalProgress {
            goal,
            assessor,
            first_score: Some(0),
            last_score: Some(0),
        })
    }
    async fn update_goal(&self, goal: Goal) -> Result<Goal, SdqError> {
        let goal_id = goal
            .goal_id
            .ok_or_else(|| SdqError::InvalidInput("Goal is missing goalId".into()))?;
        let default_type = GoalType::Unknown;
        let goal_type = serde_json::to_value(goal.r#type.as_ref().unwrap_or(&default_type))
            .map_err(|e| SdqError::InternalError(e.to_string()))?;
        let result = sqlx::query("UPDATE goal SET type = $2, description = $3 WHERE goal_id = $1")
            .bind(goal_id)
            .bind(goal_type.as_str())
            .bind(goal.description)
            .execute(&self.pool)
            .await
            .map_err(|e| SdqError::Db(e.to_string()))?;
        if result.rows_affected() == 0 {
            return Err(SdqError::InvalidInput(format!("Unknown goal: {goal_id}")));
        }
        self.get_goal(&goal_id).await
    }
    async fn get_goal(&self, goal_id: &Uuid) -> Result<Goal, SdqError> {
        let row = sqlx::query_as::<_, (Uuid, Option<String>, Option<String>)>(
            "SELECT client_id, type, description FROM goal WHERE goal_id = $1",
        )
        .bind(goal_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SdqError::Db(e.to_string()))?
        .ok_or_else(|| SdqError::InvalidInput(format!("Unknown goal: {goal_id}")))?;

        Ok(Goal {
            client_id: Some(row.0),
            goal_id: Some(*goal_id),
            r#type: row
                .1
                .and_then(|value| GoalType::from_str(&value).ok())
                .or(Some(GoalType::Unknown)),
            description: row.2,
        })
    }
    async fn submit_gbo(&self, submission: GboSubmission) -> Result<(), SdqError> {
        crate::gbo_service::GboServiceSqlxImpl::new(self.pool.clone())
            .save_gbo(submission)
            .await
    }
}
