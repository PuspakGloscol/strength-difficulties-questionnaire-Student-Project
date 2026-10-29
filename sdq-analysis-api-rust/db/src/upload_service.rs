use async_trait::async_trait;
use sdq_model::{ParsedFile, SdqError};
use sdq_service::upload::UploadService;
use sqlx::PgPool;

pub struct UploadServiceSqlxImpl {
    pub pool: PgPool,
}

impl UploadServiceSqlxImpl {
    pub fn new(pool: PgPool) -> UploadServiceSqlxImpl {
        UploadServiceSqlxImpl { pool }
    }
}

#[async_trait]
impl UploadService for UploadServiceSqlxImpl {
    async fn ingest_file(&self, file: ParsedFile) -> Result<ParsedFile, SdqError> {
        macro_rules! db_enum {
            ($value:expr) => {{
                let serialized = serde_json::to_value($value)
                    .map_err(|e| SdqError::InternalError(e.to_string()))?;
                serialized.as_str().map(str::to_owned)
            }};
        }

        let client = &file.client;
        let code_name = client
            .code_name
            .as_deref()
            .ok_or_else(|| SdqError::InvalidInput("Client is missing codeName".into()))?;
        let gender = db_enum!(&client.gender);
        let council = db_enum!(&client.council);
        let ethnicity = db_enum!(&client.ethnicity);
        let eal = db_enum!(&client.eal);
        let disability_status = db_enum!(&client.disability_status);
        let care_experience = db_enum!(&client.care_experience);
        let funding_source = db_enum!(&client.funding_source);

        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|e| SdqError::Db(e.to_string()))?;
        sqlx::query(
            r#"
            INSERT INTO client
                (client_id, code_name, date_of_birth, gender, council, ethnicity, eal,
                 disability_status, care_experience, funding_source)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(client.client_id)
        .bind(code_name)
        .bind(client.date_of_birth)
        .bind(gender.as_deref())
        .bind(council.as_deref())
        .bind(ethnicity.as_deref())
        .bind(eal.as_deref())
        .bind(disability_status.as_deref())
        .bind(care_experience.as_deref())
        .bind(funding_source.as_deref())
        .execute(&mut *transaction)
        .await
        .map_err(|e| SdqError::Db(e.to_string()))?;

        for disability_type in &client.disability_types {
            let value = db_enum!(disability_type)
                .ok_or_else(|| SdqError::InternalError("Invalid disability type".into()))?;
            sqlx::query("INSERT INTO disability_type (client_id, disability_type) VALUES ($1, $2)")
                .bind(client.client_id)
                .bind(value)
                .execute(&mut *transaction)
                .await
                .map_err(|e| SdqError::Db(e.to_string()))?;
        }
        for intervention in &client.interventions {
            sqlx::query(
                "INSERT INTO intervention_type (client_id, intervention_type, sessions) VALUES ($1, $2, $3)",
            )
            .bind(client.client_id)
            .bind(&intervention.r#type)
            .bind(intervention.sessions)
            .execute(&mut *transaction)
            .await
            .map_err(|e| SdqError::Db(e.to_string()))?;
        }
        for (ace_type, score) in &client.aces {
            let value = db_enum!(ace_type)
                .ok_or_else(|| SdqError::InternalError("Invalid ACE type".into()))?;
            sqlx::query("INSERT INTO aces (client_id, ace_type, score) VALUES ($1, $2, $3)")
                .bind(client.client_id)
                .bind(value)
                .bind(score)
                .execute(&mut *transaction)
                .await
                .map_err(|e| SdqError::Db(e.to_string()))?;
        }

        for goal in &file.goals {
            let client_id = goal
                .client_id
                .ok_or_else(|| SdqError::InvalidInput("Goal is missing clientId".into()))?;
            let goal_id = goal
                .goal_id
                .ok_or_else(|| SdqError::InvalidInput("Goal is missing goalId".into()))?;
            let default_type = sdq_model::GoalType::Unknown;
            let goal_type = db_enum!(goal.r#type.as_ref().unwrap_or(&default_type));
            sqlx::query(
                "INSERT INTO goal (client_id, goal_id, type, description) VALUES ($1, $2, $3, $4)",
            )
            .bind(client_id)
            .bind(goal_id)
            .bind(goal_type.as_deref())
            .bind(&goal.description)
            .execute(&mut *transaction)
            .await
            .map_err(|e| SdqError::Db(e.to_string()))?;
        }

        for reporting_period in &file.sdq {
            let period = &reporting_period.period;
            sqlx::query(
                "INSERT INTO reporting_period (period_id, client_id, period_date) VALUES ($1, $2, $3)",
            )
            .bind(period.period_id)
            .bind(period.client_id)
            .bind(period.period)
            .execute(&mut *transaction)
            .await
            .map_err(|e| SdqError::Db(e.to_string()))?;

            for submission in reporting_period.sdq.values() {
                for score in &submission.scores {
                    sqlx::query(
                        "INSERT INTO sdq (period_id, assessor, statement, score) VALUES ($1, $2, $3, $4)",
                    )
                    .bind(submission.period_id)
                    .bind(submission.assessor.to_string())
                    .bind(&score.statement.key)
                    .bind(score.score)
                    .execute(&mut *transaction)
                    .await
                    .map_err(|e| SdqError::Db(e.to_string()))?;
                }
            }
        }

        for submission in &file.gbo {
            let goal_id = submission
                .gbo_id
                .ok_or_else(|| SdqError::InvalidInput("GBO submission is missing goalId".into()))?;
            let period = submission
                .period
                .ok_or_else(|| SdqError::InvalidInput("GBO submission is missing period".into()))?;
            let assessor = submission.assessor.as_ref().ok_or_else(|| {
                SdqError::InvalidInput("GBO submission is missing assessor".into())
            })?;
            sqlx::query(
                "INSERT INTO gbo_score (goal_id, period_date, assessor, score) VALUES ($1, $2, $3, $4)",
            )
            .bind(goal_id)
            .bind(period)
            .bind(assessor.to_string())
            .bind(submission.score)
            .execute(&mut *transaction)
            .await
            .map_err(|e| SdqError::Db(e.to_string()))?;
        }

        transaction
            .commit()
            .await
            .map_err(|e| SdqError::Db(e.to_string()))?;
        Ok(file)
    }
}
