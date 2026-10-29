use async_trait::async_trait;
use sdq_model::{Category, Posture, SdqError, Statement};
use sdq_service::statement::StatementService;
use sqlx::PgPool;
use std::str::FromStr;

pub struct StatementServiceSqlxImpl {
    pub pool: PgPool,
}

impl StatementServiceSqlxImpl {
    pub fn new(pool: PgPool) -> StatementServiceSqlxImpl {
        StatementServiceSqlxImpl { pool }
    }
}

#[async_trait]
impl StatementService for StatementServiceSqlxImpl {
    async fn get_statement(&self, key: String) -> Result<Statement, SdqError> {
        let row = sqlx::query_as::<_, (i64, String, String, String, Option<String>, bool)>(
            r#"
            SELECT s."order"::BIGINT, s.statement_key, s.category, c.posture,
                   s.description, s.is_true_positive
            FROM sdq_statement s
            JOIN sdq_category c ON c.category = s.category
            WHERE s.statement_key = $1
            "#,
        )
        .bind(&key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SdqError::Db(e.to_string()))?
        .ok_or_else(|| SdqError::InvalidInput(format!("Unknown statement: {key}")))?;

        Ok(Statement {
            order: row.0,
            key: row.1,
            category: Category {
                category: row.2,
                posture: Posture::from_str(&row.3).unwrap_or(Posture::Unknown),
            },
            description: row.4,
            is_true_positive: row.5,
        })
    }

    async fn get_statements(&self) -> Result<Vec<Statement>, SdqError> {
        let rows = sqlx::query_as::<_, (i64, String, String, String, Option<String>, bool)>(
            r#"
            SELECT s."order"::BIGINT, s.statement_key, s.category, c.posture,
                   s.description, s.is_true_positive
            FROM sdq_statement s
            JOIN sdq_category c ON c.category = s.category
            ORDER BY s."order" ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SdqError::Db(e.to_string()))?;

        rows.into_iter()
            .map(|row| {
                Ok(Statement {
                    order: row.0,
                    key: row.1,
                    category: Category {
                        category: row.2,
                        posture: Posture::from_str(&row.3).unwrap_or(Posture::Unknown),
                    },
                    description: row.4,
                    is_true_positive: row.5,
                })
            })
            .collect()
    }
    async fn get_categories(&self) -> Result<Vec<Category>, SdqError> {
        let rows = sqlx::query_as::<_, (String, String)>(
            "SELECT category, posture FROM sdq_category ORDER BY category",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SdqError::Db(e.to_string()))?;

        rows.into_iter()
            .map(|(category, posture)| {
                Ok(Category {
                    category,
                    posture: Posture::from_str(&posture).unwrap_or(Posture::Unknown),
                })
            })
            .collect()
    }
}
