use crate::{
    errors::AppError,
    repo::link_repo::Link,
    state::AppState,
};
use askama::Template;
// Use UFCS for askama_axum::IntoResponse to avoid trait import conflicts
use axum::{
    extract::{Path, State},
    response::Response,
    routing::{get, post, put},
    Form, Router,
};
use axum::http::HeaderMap;
use axum::body::Bytes;
// Accept both JSON and form bodies using two handlers
use serde::Deserialize;
use axum::response::Redirect;

pub fn frogol_routes() -> Router<AppState> {
    use axum::middleware::from_fn;
    let api = Router::new()
        .route("/api/frogol/:slug/links", post(add_link))
        .route("/api/links/order", put(update_link_order_any))
        .route("/api/links/:id", get(show_link_fragment).put(update_link).delete(delete_link))
        .route("/api/links/:id/edit", get(edit_link_form))
        .route("/api/links/:id/click", get(track_link_click).post(track_link_click))
        .route_layer(from_fn(crate::middleware::csrf::csrf_verify));

    Router::new()
        .route("/:slug", get(show_frogol))
        .merge(api)
}

#[derive(Template)]
#[template(path = "frogol.html")]
struct FrogolPageTemplate<'a> {
    frogol_id: &'a str,
    slug: &'a str,
    display_name: &'a str,
    links: &'a Vec<Link>,
    theme: &'a str,
    avatar_url: Option<&'a str>,
    bio: Option<&'a str>,
}

#[derive(Template)]
#[template(path = "partials/links.html")]
struct LinksFragmentTemplate<'a> {
    links: &'a Vec<Link>,
}

#[derive(Template)]
#[template(path = "partials/link.html")]
struct LinkFragmentTemplate<'a> {
    link: &'a Link,
}

#[derive(Template)]
#[template(path = "partials/edit-link-form.html")]
struct EditLinkFragmentTemplate<'a> {
    link: &'a Link,
}

#[derive(Template)]
#[template(path = "dashboard/partials/link.html")]
struct DashboardLinkFragmentTemplate<'a> {
    link: &'a Link,
    clicks: i64,
}

#[derive(Template)]
#[template(path = "dashboard/partials/edit-link-form.html")]
struct DashboardEditLinkFragmentTemplate<'a> {
    link: &'a Link,
}

#[derive(Deserialize)]
pub struct AddLinkForm {
    pub url: String,
    pub label: String,
}

#[derive(Deserialize)]
struct UpdateLinkForm {
    url: Option<String>,
    label: Option<String>,
    #[serde(default)]
    is_active: Option<bool>,
}



async fn show_frogol(
    Path(slug): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let frogol = state.services.frogol.get_by_slug(&slug).await?;
    let links = state.services.frogol.get_links(&frogol.id).await?;

    if headers.contains_key("HX-Request") {
        let template = LinksFragmentTemplate { links: &links };
        Ok(<LinksFragmentTemplate as askama_axum::IntoResponse>::into_response(template))
    } else {
        let template = FrogolPageTemplate {
            frogol_id: &frogol.id,
            slug: &frogol.slug,
            display_name: frogol.display_name.as_deref().unwrap_or(""),
            links: &links,
            theme: frogol.theme.as_deref().unwrap_or("default"),
            avatar_url: frogol.avatar_url.as_deref(),
            bio: frogol.bio.as_deref(),
        };
        Ok(<FrogolPageTemplate as askama_axum::IntoResponse>::into_response(template))
    }
}

async fn add_link(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    headers: HeaderMap,
    Form(form): Form<AddLinkForm>,
) -> Result<Response, AppError> {
    let frogol = state.services.frogol.get_by_slug(&slug).await?;
    let link = state
        .services
        .frogol
        .add_link(&frogol.id, &form.url, &form.label)
        .await?;

    // If not an HTMX request (e.g., from dashboard form), redirect back to dashboard detail
    if !headers.contains_key("HX-Request") {
        return Ok(axum::response::IntoResponse::into_response(
            Redirect::to(&format!("/dashboard/frogol/{}", frogol.id))
        ));
    }
    // If dashboard view requested, return dashboard link row
    if headers
        .get("X-View")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("dashboard"))
        .unwrap_or(false)
    {
        let clicks: i64 = 0;
        let tmpl = DashboardLinkFragmentTemplate { link: &link, clicks };
        return Ok(<DashboardLinkFragmentTemplate as askama_axum::IntoResponse>::into_response(tmpl));
    }
    let template = LinkFragmentTemplate { link: &link };
    Ok(<LinkFragmentTemplate as askama_axum::IntoResponse>::into_response(template))
}

async fn update_link_order_any(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, AppError> {
    let content_type = headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Accept multiple JSON shapes and form-encoded
    let link_ids: Vec<String> = if content_type.to_ascii_lowercase().starts_with("application/json") {
        // Try { "id": ["..."] } first
        if let Ok(obj) = serde_json::from_slice::<serde_json::Value>(&body) {
            match obj {
                serde_json::Value::Object(map) => {
                    if let Some(serde_json::Value::Array(arr)) = map.get("id") {
                        arr.into_iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    } else if let Some(serde_json::Value::Array(arr)) = map.get("ids") {
                        arr.into_iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    } else {
                        return Err(AppError::InvalidInput("Invalid JSON shape".into()));
                    }
                }
                serde_json::Value::Array(arr) => arr
                    .into_iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect(),
                _ => return Err(AppError::InvalidInput("Invalid JSON".into())),
            }
        } else {
            return Err(AppError::InvalidInput("Invalid JSON".into()));
        }
    } else {
        // Handle form-encoded data: id=value1&id=value2&id=value3
        let form_str = String::from_utf8(body.to_vec())
            .map_err(|_| AppError::InvalidInput("Invalid form encoding".into()))?;
        
        let mut link_ids = Vec::new();
        for pair in form_str.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                if key == "id" {
                    if let Ok(decoded) = urlencoding::decode(value) {
                        link_ids.push(decoded.to_string());
                    }
                }
            }
        }
        link_ids
    };

    if !link_ids.is_empty() {
        tracing::info!(count = link_ids.len(), "update_link_order_any: received link ids");
        state.services.frogol.update_link_order(&link_ids).await?;
    } else {
        tracing::warn!("update_link_order_any: received empty link id list");
    }
    Ok(Response::new("".to_string().into()))
}

//

async fn edit_link_form(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let link = state.services.frogol.get_link(&id).await?;
    // If dashboard view requested, render dashboard edit fragment
    if headers
        .get("X-View")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("dashboard"))
        .unwrap_or(false)
    {
        let tmpl = DashboardEditLinkFragmentTemplate { link: &link };
        return Ok(<DashboardEditLinkFragmentTemplate as askama_axum::IntoResponse>::into_response(tmpl));
    }
    let template = EditLinkFragmentTemplate { link: &link };
    Ok(<EditLinkFragmentTemplate as askama_axum::IntoResponse>::into_response(template))
}

async fn show_link_fragment(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let link = state.services.frogol.get_link(&id).await?;
    if headers
        .get("X-View")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("dashboard"))
        .unwrap_or(false)
    {
        // We need clicks; fetch per frogol is more efficient, but we only have link here
        // Use a small helper: get click stats for this link via repo.
        // For simplicity, call get_clicks_by_link on the frogol id after fetching link
        let frogol_id = link.frogol_id.clone();
        let clicks_map = state.services.frogol.get_clicks_by_link(&frogol_id).await?;
        let clicks = *clicks_map.get(&link.id).unwrap_or(&0);
        let tmpl = DashboardLinkFragmentTemplate { link: &link, clicks };
        return Ok(<DashboardLinkFragmentTemplate as askama_axum::IntoResponse>::into_response(tmpl));
    }
    let template = LinkFragmentTemplate { link: &link };
    Ok(<LinkFragmentTemplate as askama_axum::IntoResponse>::into_response(template))
}

async fn update_link(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Form(form): Form<UpdateLinkForm>,
) -> Result<Response, AppError> {
    // Toggle visibility if requested first
    if let Some(active) = form.is_active {
        state.services.frogol.set_link_active(&id, active).await?;
        if !active {
            // Removing element on client via hx-swap="outerHTML"
            return Ok(Response::new("".into()));
        }
    }

    // If url/label not provided, keep existing
    let existing = state.services.frogol.get_link(&id).await?;
    let new_url = form.url.as_deref().unwrap_or(&existing.url).to_string();
    let new_label = form.label.as_deref().unwrap_or(&existing.label).to_string();
    let link = if form.url.is_some() || form.label.is_some() {
        state.services.frogol.update_link(&id, &new_url, &new_label).await?
    } else {
        existing
    };
    if headers
        .get("X-View")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("dashboard"))
        .unwrap_or(false)
    {
        let clicks_map = state.services.frogol.get_clicks_by_link(&link.frogol_id).await?;
        let clicks = *clicks_map.get(&link.id).unwrap_or(&0);
        let tmpl = DashboardLinkFragmentTemplate { link: &link, clicks };
        return Ok(<DashboardLinkFragmentTemplate as askama_axum::IntoResponse>::into_response(tmpl));
    }
    let template = LinkFragmentTemplate { link: &link };
    Ok(<LinkFragmentTemplate as askama_axum::IntoResponse>::into_response(template))
}

async fn delete_link(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    state.services.frogol.delete_link(&id).await?;
    Ok(Response::new("".to_string().into()))
}

async fn track_link_click(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Track the click (no IP from headers; can be extended with X-Forwarded-For)
    state
        .services
        .frogol
        .track_click(&id, None, user_agent)
        .await?;

    // Get the link to redirect
    let link = state.services.frogol.get_link(&id).await?;

    // Redirect to the actual URL
    Ok(axum::response::IntoResponse::into_response(
        axum::response::Redirect::to(&link.url)
    ))
}
