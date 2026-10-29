use async_trait::async_trait;
use sdq_model::{GboSubmission, SdqError};
use sdq_service::gbo::GboService;
use sqlx::PgPool;

pub struct GboServiceSqlxImpl {
    pub pool: PgPool,
}

impl GboServiceSqlxImpl {
    pub fn new(pool: PgPool) -> GboServiceSqlxImpl {
        GboServiceSqlxImpl { pool }
    }
}

#[async_trait]
impl GboService for GboServiceSqlxImpl {
    async fn save_gbo(&self, submission: GboSubmission) -> Result<(), SdqError> {
        let goal_id = submission
            .gbo_id
            .ok_or_else(|| SdqError::InvalidInput("GBO submission is missing goalId".into()))?;
        let period = submission
            .period
            .ok_or_else(|| SdqError::InvalidInput("GBO submission is missing period".into()))?;
        let assessor = submission
            .assessor
            .ok_or_else(|| SdqError::InvalidInput("GBO submission is missing assessor".into()))?;

        sqlx::query(
            r#"
            INSERT INTO gbo_score (goal_id, period_date, assessor, score)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(goal_id)
        .bind(period)
        .bind(assessor.to_string())
        .bind(submission.score)
        .execute(&self.pool)
        .await
        .map_err(|e| SdqError::Db(e.to_string()))
        .map(|_| ())
    }
    async fn delete_all_gbos(&self) -> Result<(), SdqError> {
        sqlx::query("DELETE FROM gbo_score")
            .execute(&self.pool)
            .await
            .map_err(|e| SdqError::Db(e.to_string()))
            .map(|_| ())
    }
}
