use crate::repo::lead_repo::{LeadRepo, NewLead, Lead, LeadSummary};
use crate::errors::AppError;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug)]
pub struct LeadService {
    repo: Arc<LeadRepo>,
}

impl LeadService {
    pub fn new(repo: Arc<LeadRepo>) -> Self {
        Self { repo }
    }

    pub async fn capture_lead(
        &self,
        frogol_id: &str,
        email: &str,
        source: Option<&str>,
        message: Option<&str>,
    ) -> Result<Lead, AppError> {
        if !email.contains('@') {
            return Err(AppError::InvalidInput("Invalid email format".to_string()));
        }
        let score = self.calculate_lead_score(source);
        let new_lead = NewLead {
            id: Uuid::new_v4().to_string(),
            frogol_id: frogol_id.to_string(),
            email: email.to_string(),
            source: source.map(|s| s.to_string()),
            score: Some(score),
            message: message.map(|m| m.to_string()),
        };
        self.repo.create_lead(new_lead).await
    }

    pub async fn get_frogol_leads(&self, frogol_id: &str) -> Result<Vec<LeadSummary>, AppError> {
        self.repo.get_frogol_leads(frogol_id).await
    }

    pub async fn get_user_total_leads(&self, user_id: &str) -> Result<i64, AppError> {
        self.repo.get_user_total_leads(user_id).await
    }

    pub async fn get_lead(&self, lead_id: &str) -> Result<Lead, AppError> {
        self.repo.get_lead(lead_id).await
    }

    pub async fn update_lead(
        &self,
        lead_id: &str,
        email: &str,
        source: Option<&str>,
        score: Option<i64>,
        message: Option<&str>,
    ) -> Result<Lead, AppError> {
        if !email.contains('@') {
            return Err(AppError::InvalidInput("Invalid email format".to_string()));
        }
        self.repo.update_lead(lead_id, email, source, score, message).await
    }

    pub async fn delete_lead(&self, lead_id: &str) -> Result<(), AppError> {
        self.repo.delete_lead(lead_id).await
    }

    fn calculate_lead_score(&self, source: Option<&str>) -> i64 {
        match source {
            Some("direct") => 100,
            Some("social") => 80,
            Some("referral") => 90,
            _ => 70,
        }
    }
}
