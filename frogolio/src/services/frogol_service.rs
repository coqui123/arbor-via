use crate::{
    errors::AppError,
    repo::{
        frogol_repo::{Frogol, FrogolRepo, NewFrogol, FrogolSummary, UserAnalytics},
        link_repo::{Link, LinkRepo, NewLink},
        click_repo::ClickRepo,
    },
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug)]
pub struct FrogolService {
    frogol_repo: Arc<FrogolRepo>,
    link_repo: Arc<LinkRepo>,
    click_repo: Arc<ClickRepo>,
}

impl FrogolService {
    pub fn new(frogol_repo: Arc<FrogolRepo>, link_repo: Arc<LinkRepo>) -> Self {
        let pool = frogol_repo.get_pool().clone();
        Self {
            frogol_repo,
            link_repo,
            click_repo: Arc::new(ClickRepo::new(pool)),
        }
    }

    pub async fn create_frogol(
        &self,
        user_id: &str,
        slug: &str,
        display_name: &str,
    ) -> Result<Frogol, AppError> {
        // Sanitize and validate slug
        let sanitized = Self::sanitize_slug(slug)?;
        // Ensure unique slug for better UX (DB also enforces UNIQUE)
        if let Ok(_) = self.frogol_repo.get_by_slug(&sanitized).await {
            return Err(AppError::InvalidInput("Slug already exists".to_string()));
        }
        let new_frogol = NewFrogol {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            slug: sanitized,
            display_name: Some(display_name.to_string()),
        };
        self.frogol_repo.create_frogol(new_frogol).await
    }

    pub async fn get_by_slug(&self, slug: &str) -> Result<Frogol, AppError> {
        self.frogol_repo.get_by_slug(slug).await
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Frogol, AppError> {
        self.frogol_repo.get_by_id(id).await
    }

    pub async fn get_user_frogols(&self, user_id: &str) -> Result<Vec<FrogolSummary>, AppError> {
        self.frogol_repo.get_user_frogols(user_id).await
    }

    pub async fn update_frogol(&self, id: &str, display_name: &str, theme: &str, avatar_url: Option<&str>, bio: Option<&str>) -> Result<Frogol, AppError> {
        self.frogol_repo.update_frogol(id, display_name, theme, avatar_url, bio).await
    }

    pub async fn update_frogol_avatar_url(&self, id: &str, avatar_url: &str) -> Result<Frogol, AppError> {
        self.frogol_repo.update_frogol_avatar_url(id, avatar_url).await
    }

    pub async fn delete_frogol(&self, id: &str) -> Result<(), AppError> {
        self.frogol_repo.delete_frogol(id).await
    }

    pub async fn add_link(
        &self,
        frogol_id: &str,
        url: &str,
        label: &str,
    ) -> Result<Link, AppError> {
        let normalized_url = Self::normalize_url(url);
        // place at end by default
        let next_order = self.link_repo.get_next_sort_order(frogol_id).await?;
        let new_link = NewLink {
            id: Uuid::new_v4().to_string(),
            frogol_id: frogol_id.to_string(),
            url: normalized_url,
            label: label.to_string(),
            sort_order: next_order,
            is_active: true,
            kind: "link".to_string(),
        };
        self.link_repo.add_link(new_link).await
    }

    pub async fn get_links(&self, frogol_id: &str) -> Result<Vec<Link>, AppError> {
        self.link_repo.get_links(frogol_id).await
    }

    pub async fn get_links_all(&self, frogol_id: &str) -> Result<Vec<Link>, AppError> {
        self.link_repo.get_links_all(frogol_id).await
    }

    pub async fn update_link_order(&self, link_ids: &[String]) -> Result<(), AppError> {
        self.link_repo.update_link_order(link_ids).await
    }

    pub async fn get_link(&self, link_id: &str) -> Result<Link, AppError> {
        self.link_repo.get_link(link_id).await
    }

    pub async fn update_link(&self, link_id: &str, url: &str, label: &str) -> Result<Link, AppError> {
        let normalized_url = Self::normalize_url(url);
        self.link_repo.update_link(link_id, &normalized_url, label).await
    }

    pub async fn delete_link(&self, link_id: &str) -> Result<(), AppError> {
        self.link_repo.delete_link(link_id).await
    }

    pub async fn set_link_active(&self, link_id: &str, active: bool) -> Result<(), AppError> {
        self.link_repo.set_link_active(link_id, active).await
    }

    pub async fn track_click(&self, link_id: &str, ip_address: Option<String>, user_agent: Option<String>) -> Result<(), AppError> {
        self.click_repo.track_click(link_id, ip_address, user_agent).await
    }

    pub async fn get_click_stats(&self, frogol_id: &str) -> Result<crate::repo::click_repo::ClickStats, AppError> {
        self.click_repo.get_frogol_click_stats(frogol_id).await
    }

    pub async fn get_user_total_clicks(&self, user_id: &str) -> Result<i64, AppError> {
        self.click_repo.get_user_total_clicks(user_id).await
    }

    pub async fn get_clicks_by_link(&self, frogol_id: &str) -> Result<std::collections::HashMap<String, i64>, AppError> {
        let pairs = self.click_repo.get_clicks_by_link(frogol_id).await?;
        Ok(pairs.into_iter().collect())
    }

    pub async fn get_user_analytics(&self, user_id: &str) -> Result<UserAnalytics, AppError> {
        self.frogol_repo.get_user_analytics(user_id).await
    }

    fn normalize_url(url: &str) -> String {
        let trimmed = url.trim();
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            return trimmed.to_string();
        }
        format!("https://{}", trimmed)
    }

    fn sanitize_slug(input: &str) -> Result<String, AppError> {
        let mut s = input.trim().to_lowercase();
        // strip protocol
        if let Some(rest) = s.strip_prefix("https://") {
            s = rest.to_string();
        } else if let Some(rest) = s.strip_prefix("http://") {
            s = rest.to_string();
        }
        // strip common prefix
        if let Some(rest) = s.strip_prefix("www.") {
            s = rest.to_string();
        }
        // take only first path segment
        if let Some(idx) = s.find('/') {
            s = s[..idx].to_string();
        }
        // replace dots with dashes
        s = s.replace('.', "-");
        // replace whitespace with dashes
        s = s.split_whitespace().collect::<Vec<_>>().join("-");
        // keep only allowed chars
        let mut cleaned = String::with_capacity(s.len());
        let mut prev_dash = false;
        for ch in s.chars() {
            let is_allowed = ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-';
            let c = if is_allowed {
                ch
            } else if ch == '_' {
                '-' // map underscore to dash
            } else {
                // drop
                continue;
            };
            let is_dash = c == '-';
            if is_dash && prev_dash {
                continue; // collapse multiple dashes
            }
            cleaned.push(c);
            prev_dash = is_dash;
        }
        let slug = cleaned.trim_matches('-').to_string();
        if slug.is_empty() {
            return Err(AppError::InvalidInput("Invalid slug".to_string()));
        }
        // Disallow reserved paths
        const RESERVED: &[&str] = &[
            "login", "logout", "register", "dashboard", "api", "static", "favicon.ico",
        ];
        if RESERVED.contains(&slug.as_str()) {
            return Err(AppError::InvalidInput("Slug is reserved".to_string()));
        }
        Ok(slug)
    }
}
