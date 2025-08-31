use sqlx::SqlitePool;
use crate::errors::AppError;
use serde::{Serialize, Deserialize};
use chrono::DateTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct Lead {
    pub id: String,
    pub frogol_id: String,
    pub email: String,
    pub source: Option<String>,
    pub score: Option<i64>,
    pub message: Option<String>,
    pub created_at: String,
    pub formatted_date: String,
}

#[derive(Debug)]
pub struct NewLead {
    pub id: String,
    pub frogol_id: String,
    pub email: String,
    pub source: Option<String>,
    pub score: Option<i64>,
    pub message: Option<String>,
}

#[derive(Debug)]
pub struct LeadRepo {
    pool: SqlitePool,
}

impl LeadRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    fn format_date(date_str: &str) -> String {
        if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
            dt.format("%b %d, %Y at %I:%M %p").to_string()
        } else {
            // Fallback to original format if parsing fails
            date_str.to_string()
        }
    }

    pub async fn create_lead(&self, new_lead: NewLead) -> Result<Lead, AppError> {
        let row = sqlx::query!(
            r#"
            INSERT INTO leads (id, frogol_id, email, source, score, message)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            RETURNING 
                id         as "id!: String",
                frogol_id  as "frogol_id!: String",
                email      as "email!: String",
                source,
                score,
                message,
                created_at as "created_at!: String"
            "#,
            new_lead.id,
            new_lead.frogol_id,
            new_lead.email,
            new_lead.source,
            new_lead.score,
            new_lead.message
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Lead {
            id: row.id,
            frogol_id: row.frogol_id,
            email: row.email,
            source: row.source,
            score: row.score,
            message: row.message,
            created_at: row.created_at.clone(),
            formatted_date: Self::format_date(&row.created_at),
        })
    }

    pub async fn get_frogol_leads(&self, frogol_id: &str) -> Result<Vec<LeadSummary>, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                id         as "id!: String",
                email      as "email!: String",
                source,
                score,
                message,
                created_at as "created_at!: String"
            FROM leads
            WHERE frogol_id = ?1
            ORDER BY created_at DESC
            "#,
            frogol_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| LeadSummary {
            id: row.id,
            email: row.email,
            source: row.source,
            score: row.score,
            message: row.message,
            created_at: row.created_at.clone(),
            formatted_date: Self::format_date(&row.created_at),
        }).collect())
    }

    pub async fn get_user_total_leads(&self, user_id: &str) -> Result<i64, AppError> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM leads l
            JOIN frogols f ON l.frogol_id = f.id
            WHERE f.user_id = ?1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    pub async fn get_lead(&self, lead_id: &str) -> Result<Lead, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT 
                id         as "id!: String",
                frogol_id  as "frogol_id!: String",
                email      as "email!: String",
                source,
                score,
                message,
                created_at as "created_at!: String"
            FROM leads
            WHERE id = ?1
            "#,
            lead_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Lead {
            id: row.id,
            frogol_id: row.frogol_id,
            email: row.email,
            source: row.source,
            score: row.score,
            message: row.message,
            created_at: row.created_at.clone(),
            formatted_date: Self::format_date(&row.created_at),
        })
    }

    pub async fn update_lead(
        &self,
        lead_id: &str,
        email: &str,
        source: Option<&str>,
        score: Option<i64>,
        message: Option<&str>,
    ) -> Result<Lead, AppError> {
        let row = sqlx::query!(
            r#"
            UPDATE leads
            SET email = ?1,
                source = ?2,
                score = ?3,
                message = ?4
            WHERE id = ?5
            RETURNING 
                id         as "id!: String",
                frogol_id  as "frogol_id!: String",
                email      as "email!: String",
                source,
                score,
                message,
                created_at as "created_at!: String"
            "#,
            email,
            source,
            score,
            message,
            lead_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Lead {
            id: row.id,
            frogol_id: row.frogol_id,
            email: row.email,
            source: row.source,
            score: row.score,
            message: row.message,
            created_at: row.created_at.clone(),
            formatted_date: Self::format_date(&row.created_at),
        })
    }

    pub async fn delete_lead(&self, lead_id: &str) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            DELETE FROM leads
            WHERE id = ?1
            "#,
            lead_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeadSummary {
    pub id: String,
    pub email: String,
    pub source: Option<String>,
    pub score: Option<i64>,
    pub message: Option<String>,
    pub created_at: String,
    pub formatted_date: String,
}
