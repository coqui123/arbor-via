use axum::{
    extract::{Path, State},
    response::Response,
    routing::{get, delete},
    Form, Router,
};
use serde::{Deserialize, Serialize};
use askama::Template;
use askama_axum::IntoResponse;
use chrono::DateTime;
use crate::{
    errors::AppError,
    state::AppState,
    repo::{
        frogol_repo::FrogolSummary,
        lead_repo::LeadSummary,
        click_repo::ClickStats,
    },
};

#[derive(Template)]
#[template(path = "dashboard/index.html")]
struct DashboardTemplate {
    user_email: String,
    frogols: Vec<FrogolSummary>,
    frogols_count: usize,
    total_leads: i64,
    total_clicks: i64,
}

#[derive(Template)]
#[template(path = "dashboard/create_frogol.html")]
struct CreateFrogolTemplate;

#[derive(Template)]
#[template(path = "dashboard/frogol.html")]
struct FrogolDetailTemplate {
    frogol: FrogolDetail,
    links: Vec<LinkDetail>,
    links_count: usize,
    leads: Vec<LeadSummary>,
    leads_count: usize,
    click_stats: ClickStats,
}

#[derive(Serialize)]
struct FrogolDetail {
    id: String,
    slug: String,
    display_name: String,
    theme: String,
    avatar_url: Option<String>,
    bio: Option<String>,
    created_at: String,
    formatted_date: String,
}

#[derive(Serialize)]
struct LinkDetail {
    id: String,
    url: String,
    label: String,
    sort_order: i32,
    clicks: i64,
    is_active: bool,
}

#[derive(Template)]
#[template(path = "dashboard/edit_frogol.html")]
struct EditFrogolTemplate {
    frogol: FrogolDetail,
}

#[derive(Template)]
#[template(path = "dashboard/analytics.html")]
struct AnalyticsTemplate {
    total_frogols: i64,
    total_links: i64,
    total_leads: i64,
    total_clicks: i64,
    top_frogols: Vec<FrogolSummary>,
}

#[derive(Template)]
#[template(path = "dashboard/settings.html")]
struct SettingsTemplate {
    user_email: String,
}

#[derive(Deserialize)]
pub struct CreateFrogolForm {
    display_name: String,
    slug: String,
}

#[derive(Deserialize)]
pub struct UpdateFrogolForm {
    display_name: String,
    theme: String,
    avatar_url: Option<String>,
    bio: Option<String>,
}

fn format_date(date_str: &str) -> String {
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        dt.format("%b %d, %Y at %I:%M %p").to_string()
    } else {
        // Fallback to original format if parsing fails
        date_str.to_string()
    }
}

pub fn dashboard_routes() -> Router<AppState> {
    Router::new()
        .route("/dashboard", get(show_dashboard))
        .route("/dashboard/frogol/new", get(show_create_frogol).post(create_frogol))
        .route("/dashboard/frogol/:id", get(show_frogol_detail))
        .route("/dashboard/frogol/:id/edit", get(show_edit_frogol).put(update_frogol))
        .route("/dashboard/frogol/:id/delete", delete(delete_frogol))
        .route("/dashboard/analytics", get(show_analytics))
        .route("/dashboard/settings", get(show_settings))
}

use tower_cookies::Cookies;
use axum::response::Redirect;

async fn show_dashboard(
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<Response, AppError> {
    // Derive user identity from auth cookie
    let token = cookies.get("auth_token").map(|c| c.value().to_string());
    let user = if let Some(token) = token {
        Some(state.services.auth.validate_token(&token).await?)
    } else {
        return Ok(Redirect::to("/login").into_response());
    };
    let user = user.expect("User should be authenticated at this point");
    let user_email = user.email.clone();
    let user_id = user.id.clone();
    
    // Get user's frogols
    let frogols = state.services.frogol.get_user_frogols(&user_id).await?;
    
    // Get total leads and clicks
    let total_leads = state.services.lead.get_user_total_leads(&user_id).await?;
    let total_clicks = state.services.frogol.get_user_total_clicks(&user_id).await?;
    
    let template = DashboardTemplate {
        user_email,
        frogols_count: frogols.len(),
        frogols,
        total_leads,
        total_clicks,
    };
    
    Ok(template.into_response())
}

async fn show_create_frogol() -> Result<Response, AppError> {
    Ok(CreateFrogolTemplate.into_response())
}

async fn create_frogol(
    State(state): State<AppState>,
    cookies: Cookies,
    Form(form): Form<CreateFrogolForm>,
) -> Result<Response, AppError> {
    // Require auth for creation; fall back to demo user email if not authenticated
    let token = cookies.get("auth_token").map(|c| c.value().to_string());
    let user = if let Some(token) = token { state.services.auth.validate_token(&token).await? } else { return Ok(Redirect::to("/login").into_response()); };
    
    let frogol = match state.services.frogol.create_frogol(&user.id, &form.slug, &form.display_name).await {
        Ok(f) => f,
        Err(AppError::InvalidInput(msg)) if msg == "Slug already exists" => {
            // Re-render the create form with a friendly message
            return Ok(axum::response::Html(format!(
                "<div class=\"max-w-xl mx-auto p-4\"><p class=\"text-red-600\">Slug already exists. Choose another.</p><a href=\"/dashboard/frogol/new\" class=\"text-indigo-600\">Back</a></div>"
            )).into_response());
        }
        Err(e) => return Err(e),
    };
    
    Ok(axum::response::Redirect::to(&format!("/dashboard/frogol/{}", frogol.id)).into_response())
}

async fn show_frogol_detail(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let frogol = state.services.frogol.get_by_id(&id).await?;
    let links = state.services.frogol.get_links_all(&id).await?;
    let clicks_by_link = state.services.frogol.get_clicks_by_link(&id).await?;
    let leads = state.services.lead.get_frogol_leads(&id).await?;
    let click_stats = state.services.frogol.get_click_stats(&id).await?;
    
    let frogol_detail = FrogolDetail {
        id: frogol.id,
        slug: frogol.slug,
        display_name: frogol.display_name.unwrap_or_else(|| "Frogol".to_string()),
        theme: frogol.theme.unwrap_or_else(|| "default".to_string()),
        avatar_url: frogol.avatar_url,
        bio: frogol.bio,
        created_at: frogol.created_at.clone(),
        formatted_date: format_date(&frogol.created_at),
    };
    
    let link_details: Vec<LinkDetail> = links.into_iter().map(|link| {
        let id = link.id;
        let clicks = *clicks_by_link.get(&id).unwrap_or(&0);
        LinkDetail {
            id,
            url: link.url,
            label: link.label,
            sort_order: link.sort_order as i32,
            clicks,
            is_active: link.is_active,
        }
    }).collect();
    
    let template = FrogolDetailTemplate {
        frogol: frogol_detail,
        links_count: link_details.len(),
        links: link_details,
        leads_count: leads.len(),
        leads,
        click_stats,
    };
    
    Ok(template.into_response())
}

async fn show_edit_frogol(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let frogol = state.services.frogol.get_by_id(&id).await?;

    let frogol_detail = FrogolDetail {
        id: frogol.id,
        slug: frogol.slug,
        display_name: frogol.display_name.unwrap_or_else(|| "Frogol".to_string()),
        theme: frogol.theme.unwrap_or_else(|| "default".to_string()),
        avatar_url: frogol.avatar_url,
        bio: frogol.bio,
        created_at: frogol.created_at.clone(),
        formatted_date: format_date(&frogol.created_at),
    };

    Ok(EditFrogolTemplate { frogol: frogol_detail }.into_response())
}

async fn update_frogol(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Form(form): Form<UpdateFrogolForm>,
) -> Result<Response, AppError> {
    let frogol = state.services.frogol.update_frogol(
        &id,
        &form.display_name,
        &form.theme,
        form.avatar_url.as_deref(),
        form.bio.as_deref(),
    ).await?;
    
    Ok(axum::response::Redirect::to(&format!("/dashboard/frogol/{}", frogol.id)).into_response())
}

async fn delete_frogol(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    state.services.frogol.delete_frogol(&id).await?;
    
    Ok(axum::response::Redirect::to("/dashboard").into_response())
}

async fn show_analytics(
    State(state): State<AppState>,
    cookies: tower_cookies::Cookies,
) -> Result<Response, AppError> {
    // Derive user identity from auth cookie
    let token = cookies.get("auth_token").map(|c| c.value().to_string());
    let user = if let Some(token) = token {
        Some(state.services.auth.validate_token(&token).await?)
    } else {
        return Ok(axum::response::Redirect::to("/login").into_response());
    };
    let user = user.expect("User should be authenticated at this point");

    let analytics = state
        .services
        .frogol
        .get_user_analytics(&user.id)
        .await?;

    let template = AnalyticsTemplate {
        total_frogols: analytics.total_frogols,
        total_links: analytics.total_links,
        total_leads: analytics.total_leads,
        total_clicks: analytics.total_clicks,
        top_frogols: analytics.top_performing_frogols,
    };

    Ok(template.into_response())
}

async fn show_settings(
    State(_state): State<AppState>,
) -> Result<Response, AppError> {
    let user_email = "user@example.com".to_string();
    Ok(SettingsTemplate { user_email }.into_response())
} 