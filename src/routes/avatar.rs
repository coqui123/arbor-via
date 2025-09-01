use axum::{
    extract::{Path, State, multipart::Multipart},
    response::Response,
    routing::post,
    Router,
};
use axum::response::IntoResponse;

use uuid::Uuid;
use crate::{
    errors::AppError,
    state::AppState,
};

pub fn avatar_routes() -> Router<AppState> {
    Router::new()
        .route("/api/frogol/:id/avatar", post(upload_avatar))
}

async fn upload_avatar(
    State(state): State<AppState>,
    Path(frogol_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Response, AppError> {
    // Find the avatar field in the multipart data
    let avatar_field = multipart.next_field().await.map_err(|e| {
        tracing::error!("Multipart error: {}", e);
        AppError::InvalidInput("Failed to process upload".to_string())
    })?.ok_or_else(|| {
        AppError::InvalidInput("No avatar field found in upload".to_string())
    })?;
    
    // Get file metadata before consuming the field
    let original_filename = avatar_field.file_name().unwrap_or("unknown").to_string();
    let content_type = avatar_field.content_type().map(|ct| ct.to_string());
    
    // Read the file data
    let file_data = avatar_field.bytes().await.map_err(|e| {
        tracing::error!("Failed to read file data: {}", e);
        AppError::Internal("Failed to read uploaded file".to_string())
    })?;
    
    // Validate file size (5MB limit)
    if file_data.len() > 5 * 1024 * 1024 {
        return Err(AppError::ValidationError("File size must be less than 5MB".to_string()));
    }
    
    // Validate content type
    let allowed_types = ["image/jpeg", "image/png", "image/gif", "image/webp"];
    if let Some(ct) = &content_type {
        if !allowed_types.contains(&ct.as_str()) {
            return Err(AppError::ValidationError("Only JPEG, PNG, GIF, and WebP images are allowed".to_string()));
        }
    }
    
    // Generate unique filename
    let extension = std::path::Path::new(&original_filename)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("bin");
    let unique_filename = format!("{}.{}", Uuid::new_v4(), extension);
    
    // Save file to avatars directory
    let avatar_path = std::path::Path::new("static/avatars").join(&unique_filename);
    tokio::fs::write(&avatar_path, &file_data).await.map_err(|e| {
        tracing::error!("Failed to save avatar file: {}", e);
        AppError::Internal("Failed to save uploaded file".to_string())
    })?;
    
    // Get the URL for the uploaded image
    let avatar_url = format!("/static/avatars/{}", unique_filename);
    
    // Update the frogol's avatar_url in the database
    state.services.frogol.update_frogol_avatar_url(&frogol_id, &avatar_url).await?;
    
    // Return the new avatar URL as JSON
    let response = serde_json::json!({
        "success": true,
        "avatar_url": avatar_url
    });
    
    Ok(axum::response::Json(response).into_response())
}
