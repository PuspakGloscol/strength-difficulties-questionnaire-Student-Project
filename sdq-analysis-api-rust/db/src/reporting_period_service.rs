use async_trait::async_trait;
use chrono::NaiveDate;
use sdq_model::{ReportingPeriod, SdqError};
use sdq_service::reporting_period::ReportingPeriodService;
use sqlx::PgPool;
use uuid::Uuid;

pub struct ReportingPeriodServiceSqlxImpl {
    pub pool: PgPool,
}

impl ReportingPeriodServiceSqlxImpl {
    pub fn new(pool: PgPool) -> ReportingPeriodServiceSqlxImpl {
        ReportingPeriodServiceSqlxImpl { pool }
    }
}

#[derive(sqlx::FromRow)]
pub struct ReportingPeriodSqlRow {
    pub period_id: Uuid,
    pub client_id: Uuid,
    pub period_date: NaiveDate,
}

impl From<ReportingPeriodSqlRow> for ReportingPeriod {
    fn from(raw: ReportingPeriodSqlRow) -> Self {
        ReportingPeriod {
            client_id: raw.client_id,
            period_id: raw.period_id,
            period: raw.period_date,
        }
    }
}

#[async_trait]
impl ReportingPeriodService for ReportingPeriodServiceSqlxImpl {
    async fn save(&self, period: ReportingPeriod) -> Result<(), SdqError> {
        sqlx::query(
            r#"
        INSERT INTO reporting_period
            (period_id, client_id, period_date)
        VALUES
            ($1, $2, $3)
        "#,
        )
        .bind(period.period_id)
        .bind(period.client_id)
        .bind(period.period) // NaiveDate maps cleanly to DATE
        .execute(&self.pool)
        .await
        .map_err(|e| SdqError::Db(e.to_string()))
        .map(|_| ())
    }

    async fn get_for_client(&self, client_id: &Uuid) -> Result<Vec<ReportingPeriod>, SdqError> {
        sqlx::query_as::<_, ReportingPeriodSqlRow>(
            r#"
        SELECT period_id, client_id, period_date
        FROM reporting_period
        WHERE client_id = $1
        "#,
        )
        .bind(client_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SdqError::Db(e.to_string()))
        .map(|raw_vec| raw_vec.into_iter().map(ReportingPeriod::from).collect())
    }

    async fn delete_all(&self) -> Result<(), SdqError> {
        sqlx::query("DELETE FROM reporting_period")
            .execute(&self.pool)
            .await
            .map_err(|e| SdqError::Db(e.to_string()))
            .map(|_| ())
    }
}
