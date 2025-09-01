use axum::{
    extract::{Path, State, Form},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use crate::state::AppState;
use crate::errors::AppError;
use askama::Template;
// askama_axum::IntoResponse is used via the trait; no direct import needed

#[derive(Template)]
#[template(path = "partials/lead-capture-success.html")]
struct LeadCaptureSuccessTemplate;

#[derive(Template)]
#[template(path = "partials/lead-capture-error.html")]
struct LeadCaptureErrorTemplate {
    error: String,
}


#[derive(Deserialize)]
pub struct LeadCaptureForm {
    email: String,
    #[allow(dead_code)]
    source: Option<String>,
    message: Option<String>,
}

pub fn lead_routes() -> Router<AppState> {
    use axum::middleware::from_fn;
    let api_csrf = from_fn(crate::middleware::csrf::csrf_verify);
    Router::new()
        .route("/api/lead/:frogol_id", post(capture_lead)).route_layer(api_csrf.clone())
        .route("/api/leads/:id", get(show_lead_fragment).put(update_lead).delete(delete_lead)).route_layer(api_csrf.clone())
        .route("/api/leads/:id/edit", get(edit_lead_form))
}

async fn capture_lead(
    Path(frogol_id): Path<String>,
    State(state): State<AppState>,
    Form(payload): Form<LeadCaptureForm>,
) -> Result<impl IntoResponse, AppError> {
    // Validate email
    if !payload.email.contains('@') {
        let template = LeadCaptureErrorTemplate {
            error: "Invalid email format".to_string(),
        };
        return Ok(template.into_response());
    }

    let lead = state
        .services
        .lead
        .capture_lead(
            &frogol_id,
            &payload.email,
            payload.source.as_deref(),
            payload.message.as_deref(),
        )
        .await;

    match lead {
        Ok(_) => {
            let template = LeadCaptureSuccessTemplate;
            Ok(template.into_response())
        }
        Err(_) => {
            let template = LeadCaptureErrorTemplate {
                error: "Failed to capture lead".to_string(),
            };
            Ok(template.into_response())
        }
    }
}

#[derive(Template)]
#[template(path = "dashboard/partials/lead.html")]
struct DashboardLeadFragmentTemplate<'a> {
    lead: &'a crate::repo::lead_repo::Lead,
}

#[derive(Template)]
#[template(path = "dashboard/partials/edit-lead-form.html")]
struct DashboardEditLeadFragmentTemplate<'a> {
    lead: &'a crate::repo::lead_repo::Lead,
}

#[derive(Deserialize)]
struct UpdateLeadForm {
    email: String,
    source: Option<String>,
    score: Option<String>,
    message: Option<String>,
}

async fn edit_lead_form(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let lead = state.services.lead.get_lead(&id).await?;
    let tmpl = DashboardEditLeadFragmentTemplate { lead: &lead };
    Ok(tmpl.into_response())
}

async fn show_lead_fragment(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let lead = state.services.lead.get_lead(&id).await?;
    let tmpl = DashboardLeadFragmentTemplate { lead: &lead };
    Ok(tmpl.into_response())
}

async fn update_lead(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Form(form): Form<UpdateLeadForm>,
) -> Result<Response, AppError> {
    let parsed_score: Option<i64> = match form.score.as_deref() {
        Some("") | None => None,
        Some(s) => s.parse::<i64>().ok(),
    };

    let updated = state
        .services
        .lead
        .update_lead(&id, &form.email, form.source.as_deref(), parsed_score, form.message.as_deref())
        .await?;
    let tmpl = DashboardLeadFragmentTemplate { lead: &updated };
    Ok(tmpl.into_response())
}

async fn delete_lead(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    state.services.lead.delete_lead(&id).await?;
    Ok(Response::new("".into()))
}
