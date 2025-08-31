use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Serialize, Deserialize)]
pub struct Click {
    pub id: String,
    pub link_id: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug)]
pub struct NewClick {
    pub id: String,
    pub link_id: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug)]
pub struct ClickRepo {
    pool: SqlitePool,
}

impl ClickRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn record_click(&self, new_click: NewClick) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            INSERT INTO clicks (id, link_id, ip_address, user_agent)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            new_click.id,
            new_click.link_id,
            new_click.ip_address,
            new_click.user_agent
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn track_click(&self, link_id: &str, ip_address: Option<String>, user_agent: Option<String>) -> Result<(), AppError> {
        let click_id = uuid::Uuid::new_v4().to_string();
        let new_click = NewClick {
            id: click_id,
            link_id: link_id.to_string(),
            ip_address,
            user_agent,
        };
        self.record_click(new_click).await
    }

    pub async fn get_frogol_click_stats(&self, frogol_id: &str) -> Result<ClickStats, AppError> {
        let total_clicks = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM clicks c
            JOIN links l ON c.link_id = l.id
            WHERE l.frogol_id = ?1
            "#,
            frogol_id
        )
        .fetch_one(&self.pool)
        .await?;

        let unique_clicks = sqlx::query_scalar!(
            r#"
            SELECT COUNT(DISTINCT c.ip_address)
            FROM clicks c
            JOIN links l ON c.link_id = l.id
            WHERE l.frogol_id = ?1 AND c.ip_address IS NOT NULL
            "#,
            frogol_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(ClickStats {
            total_clicks,
            unique_clicks,
        })
    }

    pub async fn get_user_total_clicks(&self, user_id: &str) -> Result<i64, AppError> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM clicks c
            JOIN links l ON c.link_id = l.id
            JOIN frogols f ON l.frogol_id = f.id
            WHERE f.user_id = ?1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    pub async fn get_clicks_by_link(&self, frogol_id: &str) -> Result<Vec<(String, i64)>, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT l.id as "link_id!: String", COUNT(c.id) as "clicks!: i64"
            FROM links l
            LEFT JOIN clicks c ON c.link_id = l.id
            WHERE l.frogol_id = ?1
            GROUP BY l.id
            "#,
            frogol_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| (r.link_id, r.clicks)).collect())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClickStats {
    pub total_clicks: i64,
    pub unique_clicks: i64,
}
