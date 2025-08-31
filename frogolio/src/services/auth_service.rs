use crate::repo::user_repo::{UserRepo, User, NewUser, Session, NewSession};
use crate::errors::AppError;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // user_id
    exp: u64,    // expiration time
    iat: u64,    // issued at
}

pub struct AuthService {
    user_repo: UserRepo,
    jwt_secret: String,
}

impl AuthService {
    pub fn new(user_repo: UserRepo, jwt_secret: String) -> Self {
        Self {
            user_repo,
            jwt_secret,
        }
    }

    pub async fn register(&self, email: &str, password: &str) -> Result<User, AppError> {
        // Check if user already exists
        if let Some(_) = self.user_repo.get_by_email(email).await? {
            return Err(AppError::InvalidInput("User already exists".to_string()));
        }

        // Hash password
        let password_hash = hash(password, DEFAULT_COST)
            .map_err(|_| AppError::InternalError("Failed to hash password".to_string()))?;

        // Create user
        let new_user = NewUser {
            id: Uuid::new_v4().to_string(),
            email: email.to_string(),
            password_hash,
        };

        self.user_repo.create_user(new_user).await
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<Session, AppError> {
        // Get user by email
        let user = self.user_repo.get_by_email(email).await?
            .ok_or_else(|| AppError::InvalidInput("Invalid credentials".to_string()))?;

        // Verify password
        if let Some(password_hash) = &user.password_hash {
            let is_valid = verify(password, password_hash)
                .map_err(|_| AppError::InternalError("Failed to verify password".to_string()))?;
            
            if !is_valid {
                return Err(AppError::InvalidInput("Invalid credentials".to_string()));
            }
        } else {
            return Err(AppError::InvalidInput("Invalid credentials".to_string()));
        }

        // Check if user is active
        if !user.is_active {
            return Err(AppError::InvalidInput("Account is disabled".to_string()));
        }

        // Generate JWT token
        let token = self.generate_jwt(&user.id)?;

        // Create session
        let expires_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time should be after Unix epoch")
            .as_secs() + (24 * 60 * 60); // 24 hours

        let new_session = NewSession {
            id: Uuid::new_v4().to_string(),
            user_id: user.id,
            token: token.clone(),
            expires_at: DateTime::from_timestamp(expires_at as i64, 0)
                .expect("Valid timestamp should be convertible to DateTime")
                .to_rfc3339(),
        };

        self.user_repo.create_session(new_session).await
    }

    pub async fn validate_token(&self, token: &str) -> Result<User, AppError> {
        // Verify JWT
        let claims = self.verify_jwt(token)?;

        // Check if session exists and is valid
        let session = self.user_repo.get_session_by_token(token).await?
            .ok_or_else(|| AppError::InvalidInput("Invalid session".to_string()))?;

        // Check if session is expired
        let expires_at = DateTime::parse_from_rfc3339(&session.expires_at)
            .map_err(|_| AppError::InvalidInput("Invalid session format".to_string()))?;
        
        if expires_at < Utc::now() {
            return Err(AppError::InvalidInput("Session expired".to_string()));
        }

        // Get user
        let user = self.user_repo.get_by_id(&claims.sub).await?
            .ok_or_else(|| AppError::InvalidInput("User not found".to_string()))?;

        if !user.is_active {
            return Err(AppError::InvalidInput("Account is disabled".to_string()));
        }

        Ok(user)
    }

    pub async fn logout(&self, token: &str) -> Result<(), AppError> {
        self.user_repo.delete_session(token).await
    }

    fn generate_jwt(&self, user_id: &str) -> Result<String, AppError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time should be after Unix epoch")
            .as_secs();

        let claims = Claims {
            sub: user_id.to_string(),
            exp: now + (24 * 60 * 60), // 24 hours
            iat: now,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .map_err(|_| AppError::InternalError("Failed to generate JWT".to_string()))
    }

    fn verify_jwt(&self, token: &str) -> Result<Claims, AppError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|_| AppError::InvalidInput("Invalid token".to_string()))
    }
} 