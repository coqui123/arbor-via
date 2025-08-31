use sqlx::SqlitePool;
use crate::errors::AppError;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password_hash: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug)]
pub struct NewUser {
    pub id: String,
    pub email: String,
    pub password_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub expires_at: String,
    pub created_at: String,
}

#[derive(Debug)]
pub struct NewSession {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub expires_at: String,
}

#[derive(Debug)]
pub struct UserRepo {
    pool: SqlitePool,
}

impl UserRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, new_user: NewUser) -> Result<User, AppError> {
        let row = sqlx::query!(
            r#"
            INSERT INTO users (id, email, password_hash, is_active)
            VALUES (?1, ?2, ?3, 1)
            RETURNING 
                id            as "id!: String",
                email         as "email!: String",
                password_hash,
                is_active     as "is_active!: bool",
                created_at    as "created_at!: String"
            "#,
            new_user.id,
            new_user.email,
            new_user.password_hash
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(User {
            id: row.id,
            email: row.email,
            password_hash: row.password_hash,
            is_active: row.is_active,
            created_at: row.created_at,
        })
    }

    pub async fn get_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT 
                id            as "id!: String",
                email         as "email!: String",
                password_hash,
                is_active     as "is_active!: bool",
                created_at    as "created_at!: String"
            FROM users
            WHERE email = ?1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| User {
            id: r.id,
            email: r.email,
            password_hash: r.password_hash,
            is_active: r.is_active,
            created_at: r.created_at,
        }))
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<User>, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT 
                id            as "id!: String",
                email         as "email!: String",
                password_hash,
                is_active     as "is_active!: bool",
                created_at    as "created_at!: String"
            FROM users
            WHERE id = ?1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| User {
            id: r.id,
            email: r.email,
            password_hash: r.password_hash,
            is_active: r.is_active,
            created_at: r.created_at,
        }))
    }

    pub async fn create_session(&self, new_session: NewSession) -> Result<Session, AppError> {
        let row = sqlx::query!(
            r#"
            INSERT INTO sessions (id, user_id, token, expires_at)
            VALUES (?1, ?2, ?3, ?4)
            RETURNING 
                id         as "id!: String",
                user_id    as "user_id!: String",
                token      as "token!: String",
                expires_at as "expires_at!: String",
                created_at as "created_at!: String"
            "#,
            new_session.id,
            new_session.user_id,
            new_session.token,
            new_session.expires_at
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Session {
            id: row.id,
            user_id: row.user_id,
            token: row.token,
            expires_at: row.expires_at,
            created_at: row.created_at,
        })
    }

    pub async fn get_session_by_token(&self, token: &str) -> Result<Option<Session>, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT 
                id         as "id!: String",
                user_id    as "user_id!: String",
                token      as "token!: String",
                expires_at as "expires_at!: String",
                created_at as "created_at!: String"
            FROM sessions
            WHERE token = ?1
            "#,
            token
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Session {
            id: r.id,
            user_id: r.user_id,
            token: r.token,
            expires_at: r.expires_at,
            created_at: r.created_at,
        }))
    }

    pub async fn delete_session(&self, token: &str) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            DELETE FROM sessions
            WHERE token = ?1
            "#,
            token
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
} 