use sqlx::{SqlitePool, Row};
use crate::errors::AppError;
use serde::{Serialize, Deserialize};
use chrono::DateTime;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Frogol {
    pub id: String,
    pub user_id: String,
    pub slug: String,
    pub display_name: Option<String>,
    pub theme: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub created_at: String,
}

#[derive(Debug)]
pub struct NewFrogol {
    pub id: String,
    pub user_id: String,
    pub slug: String,
    pub display_name: Option<String>,
}

#[derive(Debug)]
pub struct FrogolRepo {
    pool: SqlitePool,
}

impl FrogolRepo {
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

    pub fn get_pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn create_frogol(&self, new_frogol: NewFrogol) -> Result<Frogol, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO frogols (id, user_id, slug, display_name)
            VALUES (?1, ?2, ?3, ?4)
            RETURNING id, user_id, slug, display_name, theme, avatar_url, bio, created_at
            "#
        )
        .bind(&new_frogol.id)
        .bind(&new_frogol.user_id)
        .bind(&new_frogol.slug)
        .bind(&new_frogol.display_name)
        .fetch_one(&self.pool)
        .await?;

        Ok(Frogol {
            id: row.try_get::<String, _>("id")?,
            user_id: row.try_get::<String, _>("user_id")?,
            slug: row.try_get::<String, _>("slug")?,
            display_name: row.try_get::<Option<String>, _>("display_name")?,
            theme: row.try_get::<Option<String>, _>("theme")?,
            avatar_url: row.try_get::<Option<String>, _>("avatar_url")?,
            bio: row.try_get::<Option<String>, _>("bio")?,
            created_at: row.try_get::<String, _>("created_at")?,
        })
    }

    pub async fn get_by_slug(&self, slug: &str) -> Result<Frogol, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, slug, display_name, theme, avatar_url, bio, created_at
            FROM frogols
            WHERE slug = ?1
            "#
        )
        .bind(slug)
        .fetch_one(&self.pool)
        .await?;

        Ok(Frogol {
            id: row.try_get::<String, _>("id")?,
            user_id: row.try_get::<String, _>("user_id")?,
            slug: row.try_get::<String, _>("slug")?,
            display_name: row.try_get::<Option<String>, _>("display_name")?,
            theme: row.try_get::<Option<String>, _>("theme")?,
            avatar_url: row.try_get::<Option<String>, _>("avatar_url")?,
            bio: row.try_get::<Option<String>, _>("bio")?,
            created_at: row.try_get::<String, _>("created_at")?,
        })
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Frogol, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, slug, display_name, theme, avatar_url, bio, created_at
            FROM frogols
            WHERE id = ?1
            "#
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Frogol {
            id: row.try_get::<String, _>("id")?,
            user_id: row.try_get::<String, _>("user_id")?,
            slug: row.try_get::<String, _>("slug")?,
            display_name: row.try_get::<Option<String>, _>("display_name")?,
            theme: row.try_get::<Option<String>, _>("theme")?,
            avatar_url: row.try_get::<Option<String>, _>("avatar_url")?,
            bio: row.try_get::<Option<String>, _>("bio")?,
            created_at: row.try_get::<String, _>("created_at")?,
        })
    }

    pub async fn get_user_frogols(&self, user_id: &str) -> Result<Vec<FrogolSummary>, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                f.id           as "id!: String",
                f.slug         as "slug!: String",
                f.display_name,
                f.created_at   as "created_at!: String",
                COALESCE(COUNT(DISTINCT l.id), 0) as "total_links!: i64",
                COALESCE(COUNT(DISTINCT ld.id), 0) as "total_leads!: i64",
                COALESCE(COUNT(DISTINCT c.id), 0) as "total_clicks!: i64"
            FROM frogols f
            LEFT JOIN links l ON f.id = l.frogol_id
            LEFT JOIN leads ld ON f.id = ld.frogol_id
            LEFT JOIN clicks c ON l.id = c.link_id
            WHERE f.user_id = ?1
            GROUP BY f.id, f.slug, f.display_name, f.created_at
            ORDER BY f.created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| FrogolSummary {
            id: row.id,
            slug: row.slug,
            display_name: row.display_name.unwrap_or_else(|| "Frogol".to_string()),
            total_links: row.total_links,
            total_leads: row.total_leads,
            total_clicks: row.total_clicks,
            created_at: row.created_at.clone(),
            formatted_date: Self::format_date(&row.created_at),
        }).collect())
    }

    pub async fn update_frogol(&self, id: &str, display_name: &str, theme: &str, avatar_url: Option<&str>, bio: Option<&str>) -> Result<Frogol, AppError> {
        let row = sqlx::query(
            r#"
            UPDATE frogols 
            SET display_name = ?1, theme = ?2, avatar_url = COALESCE(?3, avatar_url), bio = COALESCE(?4, bio)
            WHERE id = ?5
            RETURNING id, user_id, slug, display_name, theme, avatar_url, bio, created_at
            "#
        )
        .bind(display_name)
        .bind(theme)
        .bind(avatar_url)
        .bind(bio)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Frogol {
            id: row.try_get::<String, _>("id")?,
            user_id: row.try_get::<String, _>("user_id")?,
            slug: row.try_get::<String, _>("slug")?,
            display_name: row.try_get::<Option<String>, _>("display_name")?,
            theme: row.try_get::<Option<String>, _>("theme")?,
            avatar_url: row.try_get::<Option<String>, _>("avatar_url")?,
            bio: row.try_get::<Option<String>, _>("bio")?,
            created_at: row.try_get::<String, _>("created_at")?,
        })
    }

    pub async fn update_frogol_avatar_url(&self, id: &str, avatar_url: &str) -> Result<Frogol, AppError> {
        let row = sqlx::query(
            r#"
            UPDATE frogols 
            SET avatar_url = ?1
            WHERE id = ?2
            RETURNING id, user_id, slug, display_name, theme, avatar_url, bio, created_at
            "#
        )
        .bind(avatar_url)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Frogol {
            id: row.try_get::<String, _>("id")?,
            user_id: row.try_get::<String, _>("user_id")?,
            slug: row.try_get::<String, _>("slug")?,
            display_name: row.try_get::<Option<String>, _>("display_name")?,
            theme: row.try_get::<Option<String>, _>("theme")?,
            avatar_url: row.try_get::<Option<String>, _>("avatar_url")?,
            bio: row.try_get::<Option<String>, _>("bio")?,
            created_at: row.try_get::<String, _>("created_at")?,
        })
    }

    pub async fn delete_frogol(&self, id: &str) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            DELETE FROM frogols
            WHERE id = ?1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_user_analytics(&self, user_id: &str) -> Result<UserAnalytics, AppError> {
        // Get total counts
        let total_frogols = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM frogols WHERE user_id = ?1"#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        let total_links = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) 
            FROM links l
            JOIN frogols f ON l.frogol_id = f.id
            WHERE f.user_id = ?1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        let total_leads = sqlx::query_scalar!(
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

        let total_clicks = sqlx::query_scalar!(
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

        // Get top performing frogols
        let top_frogols = sqlx::query!(
            r#"
            SELECT 
                f.id           as "id!: String",
                f.slug         as "slug!: String",
                f.display_name,
                f.created_at   as "created_at!: String",
                COALESCE(COUNT(DISTINCT l.id), 0) as "total_links!: i64",
                COALESCE(COUNT(DISTINCT ld.id), 0) as "total_leads!: i64",
                COALESCE(COUNT(DISTINCT c.id), 0) as "total_clicks!: i64"
            FROM frogols f
            LEFT JOIN links l ON f.id = l.frogol_id
            LEFT JOIN leads ld ON f.id = ld.frogol_id
            LEFT JOIN clicks c ON l.id = c.link_id
            WHERE f.user_id = ?1
            GROUP BY f.id, f.slug, f.display_name, f.created_at
            ORDER BY COUNT(DISTINCT c.id) DESC, COUNT(DISTINCT ld.id) DESC
            LIMIT 5
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(UserAnalytics {
            total_frogols,
            total_links,
            total_leads,
            total_clicks,
            top_performing_frogols: top_frogols.into_iter().map(|row| FrogolSummary {
                id: row.id,
                slug: row.slug,
                display_name: row.display_name.unwrap_or_else(|| "Frogol".to_string()),
                total_links: row.total_links,
                total_leads: row.total_leads,
                total_clicks: row.total_clicks,
                created_at: row.created_at.clone(),
                formatted_date: Self::format_date(&row.created_at),
            }).collect(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FrogolSummary {
    pub id: String,
    pub slug: String,
    pub display_name: String,
    pub total_links: i64,
    pub total_leads: i64,
    pub total_clicks: i64,
    pub created_at: String,
    pub formatted_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserAnalytics {
    pub total_frogols: i64,
    pub total_links: i64,
    pub total_leads: i64,
    pub total_clicks: i64,
    pub top_performing_frogols: Vec<FrogolSummary>,
}
