use sqlx::{SqlitePool, Row};
use std::collections::HashSet;
use crate::errors::AppError;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Link {
    pub id: String,
    pub frogol_id: String,
    pub url: String,
    pub label: String,
    pub sort_order: i64,
    pub is_active: bool,
    pub kind: String,
}

impl std::fmt::Display for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

#[derive(Debug)]
pub struct NewLink {
    pub id: String,
    pub frogol_id: String,
    pub url: String,
    pub label: String,
    pub sort_order: i64,
    pub is_active: bool,
    pub kind: String,
}

#[derive(Debug)]
pub struct LinkRepo {
    pool: SqlitePool,
}

impl LinkRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_next_sort_order(&self, frogol_id: &str) -> Result<i64, AppError> {
        let next = sqlx::query_scalar!(
            r#"
            SELECT COALESCE(MAX(sort_order), -1) + 1
            FROM links
            WHERE frogol_id = ?1
            "#,
            frogol_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(next)
    }

    pub async fn add_link(&self, link: NewLink) -> Result<Link, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO links (id, frogol_id, url, label, sort_order, is_active, kind)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            RETURNING id, frogol_id, url, label, sort_order, is_active, kind
            "#
        )
        .bind(&link.id)
        .bind(&link.frogol_id)
        .bind(&link.url)
        .bind(&link.label)
        .bind(link.sort_order)
        .bind(if link.is_active { 1 } else { 0 })
        .bind(&link.kind)
        .fetch_one(&self.pool)
        .await?;

        Ok(Link {
            id: row.try_get::<String, _>("id")?,
            frogol_id: row.try_get::<String, _>("frogol_id")?,
            url: row.try_get::<String, _>("url")?,
            label: row.try_get::<String, _>("label")?,
            sort_order: row.try_get::<i64, _>("sort_order")?,
            is_active: row.try_get::<i64, _>("is_active")? != 0,
            kind: row.try_get::<String, _>("kind")?,
        })
    }

    pub async fn get_links(&self, frogol_id: &str) -> Result<Vec<Link>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, frogol_id, url, label, sort_order, is_active, kind
            FROM links
            WHERE frogol_id = ?1 AND is_active = 1
            ORDER BY sort_order, id
            "#
        )
        .bind(frogol_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| Link {
            id: row.try_get::<String, _>("id").expect("id column should exist and be a string"),
            frogol_id: row.try_get::<String, _>("frogol_id").expect("frogol_id column should exist and be a string"),
            url: row.try_get::<String, _>("url").expect("url column should exist and be a string"),
            label: row.try_get::<String, _>("label").expect("label column should exist and be a string"),
            sort_order: row.try_get::<i64, _>("sort_order").expect("sort_order column should exist and be an integer"),
            is_active: row.try_get::<i64, _>("is_active").unwrap_or(1) != 0,
            kind: row.try_get::<String, _>("kind").unwrap_or_else(|_| "link".to_string()),
        }).collect())
    }

    pub async fn get_links_all(&self, frogol_id: &str) -> Result<Vec<Link>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, frogol_id, url, label, sort_order, is_active, kind
            FROM links
            WHERE frogol_id = ?1
            ORDER BY sort_order, id
            "#
        )
        .bind(frogol_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| Link {
            id: row.try_get::<String, _>("id").expect("id column should exist and be a string"),
            frogol_id: row.try_get::<String, _>("frogol_id").expect("frogol_id column should exist and be a string"),
            url: row.try_get::<String, _>("url").expect("url column should exist and be a string"),
            label: row.try_get::<String, _>("label").expect("label column should exist and be a string"),
            sort_order: row.try_get::<i64, _>("sort_order").expect("sort_order column should exist and be an integer"),
            is_active: row.try_get::<i64, _>("is_active").unwrap_or(1) != 0,
            kind: row.try_get::<String, _>("kind").unwrap_or_else(|_| "link".to_string()),
        }).collect())
    }

    pub async fn update_link_order(&self, link_ids: &[String]) -> Result<(), AppError> {
        if link_ids.is_empty() {
            return Ok(());
        }

        // Determine the frogol_id from the first link id
        let frogol_id = sqlx::query_scalar!(
            r#"SELECT frogol_id FROM links WHERE id = ?1"#,
            link_ids[0]
        )
        .fetch_one(&self.pool)
        .await?;

        // Fetch all current link ids for this frogol in their existing order
        let existing = sqlx::query!(
            r#"
            SELECT id as "id!: String"
            FROM links
            WHERE frogol_id = ?1 AND is_active = 1
            ORDER BY sort_order, id
            "#,
            frogol_id
        )
        .fetch_all(&self.pool)
        .await?;

        // Build a complete ordered list: provided ids first, then any remaining
        let mut seen: HashSet<String> = HashSet::with_capacity(existing.len());
        let mut final_order: Vec<String> = Vec::with_capacity(existing.len());

        for lid in link_ids {
            if seen.insert(lid.clone()) {
                final_order.push(lid.clone());
            }
        }
        for row in existing {
            let id = row.id; // String
            if seen.insert(id.clone()) {
                final_order.push(id);
            }
        }

        let mut tx = self.pool.begin().await?;
        for (i, link_id) in final_order.iter().enumerate() {
            let sort_order = i as i64;
            sqlx::query!(
                "UPDATE links SET sort_order = ?1 WHERE id = ?2",
                sort_order,
                link_id
            )
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn get_link(&self, link_id: &str) -> Result<Link, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, frogol_id, url, label, sort_order, is_active, kind
            FROM links
            WHERE id = ?1
            "#
        )
        .bind(link_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Link {
            id: row.try_get::<String, _>("id")?,
            frogol_id: row.try_get::<String, _>("frogol_id")?,
            url: row.try_get::<String, _>("url")?,
            label: row.try_get::<String, _>("label")?,
            sort_order: row.try_get::<i64, _>("sort_order")?,
            is_active: row.try_get::<i64, _>("is_active")? != 0,
            kind: row.try_get::<String, _>("kind")?,
        })
    }

    pub async fn update_link(&self, link_id: &str, url: &str, label: &str) -> Result<Link, AppError> {
        let row = sqlx::query(
            r#"
            UPDATE links
            SET url = ?1, label = ?2
            WHERE id = ?3
            RETURNING id, frogol_id, url, label, sort_order, is_active, kind
            "#
        )
        .bind(url)
        .bind(label)
        .bind(link_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Link {
            id: row.try_get::<String, _>("id")?,
            frogol_id: row.try_get::<String, _>("frogol_id")?,
            url: row.try_get::<String, _>("url")?,
            label: row.try_get::<String, _>("label")?,
            sort_order: row.try_get::<i64, _>("sort_order")?,
            is_active: row.try_get::<i64, _>("is_active")? != 0,
            kind: row.try_get::<String, _>("kind")?,
        })
    }

    pub async fn set_link_active(&self, link_id: &str, active: bool) -> Result<(), AppError> {
        sqlx::query("UPDATE links SET is_active = ?1 WHERE id = ?2")
            .bind(if active { 1 } else { 0 })
            .bind(link_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_link(&self, link_id: &str) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            DELETE FROM links
            WHERE id = ?1
            "#,
            link_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
