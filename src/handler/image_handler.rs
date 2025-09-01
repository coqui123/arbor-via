use axum_typed_multipart::FieldData;
use tempfile::NamedTempFile;
use tokio::fs;
use tokio::io::AsyncReadExt;
use uuid::Uuid;
use std::path::PathBuf;
use sqlx::SqlitePool;
use infer;
use futures::stream::{self, StreamExt};

use crate::errors::AppError;
use crate::models::avatar_image::FrogolAvatarImage;

pub const ALLOWED_IMAGE_TYPES: [&str; 4] = ["image/jpeg", "image/png", "image/gif", "image/webp"];

// Struct to hold processed image data before saving to DB
pub struct ProcessedImageData {
    pub new_image_id: String,
    pub unique_filename: String,
    pub image_order: i64,
}

/// Handles processing and saving a single uploaded image.
/// Validates MIME type, generates a unique filename, saves the file,
/// and prepares data for database insertion.
pub async fn process_and_save_image(
    image_field: FieldData<NamedTempFile>,
    image_save_dir: &PathBuf,
    image_order: i64, // Used to determine the order if multiple images are uploaded
) -> Result<ProcessedImageData, AppError> {
    let original_file_name = image_field.metadata.file_name.as_ref()
        .map(|name| name.as_str())
        .unwrap_or("unknown_image.bin");
    tracing::debug!("Processing image: {}", original_file_name);

    let temp_file: NamedTempFile = image_field.contents;
    let client_content_type = image_field.metadata.content_type.as_ref();

    // MIME type validation
    let mut effective_mime_type: Option<String> = None;
    if let Some(ct_str) = client_content_type.map(|ct| ct.as_str()) {
        if ct_str != "application/octet-stream" && !ct_str.is_empty() {
            effective_mime_type = Some(ct_str.to_string());
        }
    }

    if effective_mime_type.is_none() {
        let mut file_bytes = Vec::new();
        let temp_file_path = temp_file.path().to_path_buf();
        let mut file_for_inference = tokio::fs::File::open(&temp_file_path).await.map_err(|e| {
            tracing::error!("Failed to open temp file for inference: {} (path: {:?})", e, temp_file_path);
            AppError::ValidationError("Failed to process uploaded image for type checking.".to_string())
        })?;
        file_for_inference.read_to_end(&mut file_bytes).await.map_err(|e| {
            tracing::error!("Failed to read temp file for inference: {} (path: {:?})", e, temp_file_path);
            AppError::ValidationError("Failed to read uploaded image for type checking.".to_string())
        })?;
        if let Some(kind) = infer::get(&file_bytes) {
            effective_mime_type = Some(kind.mime_type().to_string());
            tracing::info!("Inferred image type for {}: {}", original_file_name, kind.mime_type());
        } else {
            tracing::warn!("Could not infer image type for {}", original_file_name);
            return Err(AppError::ValidationError("Could not determine image type. Please upload a valid image.".to_string()));
        }
    }

    if let Some(mime_to_check) = &effective_mime_type {
        if !ALLOWED_IMAGE_TYPES.contains(&mime_to_check.as_str()) {
            tracing::warn!("Uploaded image {} has unsupported type: {} (Client: {:?})", original_file_name, mime_to_check, client_content_type.map(|c|c.to_string()));
            return Err(AppError::ValidationError(format!("Unsupported image type: {}. Only JPEG, PNG, GIF, and WebP are allowed.", mime_to_check)));
        }
    } else {
        tracing::warn!("Image type for {} remains undetermined after checks. Client type: {:?}", original_file_name, client_content_type.map(|c|c.to_string()));
        return Err(AppError::ValidationError("Image content type could not be verified. Please upload a valid image.".to_string()));
    }

    let extension = std::path::Path::new(&original_file_name)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("bin");
    let unique_filename = format!("{}.{}", Uuid::new_v4(), extension);

    fs::create_dir_all(image_save_dir).await.map_err(|e| {
        tracing::error!("Failed to create image save directory {:?}: {}", image_save_dir, e);
        AppError::Internal("Failed to prepare image storage.".to_string())
    })?;
    let image_save_path = image_save_dir.join(&unique_filename);
    let temp_file_path_for_copy = temp_file.path().to_path_buf();

    tokio::fs::copy(&temp_file_path_for_copy, &image_save_path).await.map_err(|e| {
        tracing::error!("Failed to copy temp file {} to {}: {:#}", temp_file_path_for_copy.display(), image_save_path.display(), e);
        AppError::Internal("Failed to save uploaded image.".to_string())
    })?;

    let new_image_id = Uuid::new_v4().to_string();

    Ok(ProcessedImageData {
        new_image_id,
        unique_filename,
        image_order,
    })
}

/// Deletes an image file from the filesystem.
pub async fn delete_image_file(image_filename: &str, image_save_dir: &PathBuf) -> Result<(), AppError> {
    let image_path_to_delete = image_save_dir.join(image_filename);
    if image_path_to_delete.exists() {
        tokio::fs::remove_file(&image_path_to_delete).await.map_err(|e| {
            tracing::warn!("Failed to delete image file {}: {}", image_filename, e);
            AppError::Internal(format!("Failed to delete image file: {}", image_filename))
        })?;
        tracing::info!("Deleted image file: {}", image_filename);
    } else {
        tracing::warn!("Image file {} not found for deletion.", image_filename);
        // Depending on strictness, you might return an error here or just log
    }
    Ok(())
}

/// Deletes image metadata from the database for a specific avatar image of a frogol.
pub async fn delete_avatar_image_metadata_from_db(
    pool: &SqlitePool,
    frogol_id: &str,
    image_filename: &str,
) -> Result<u64, AppError> {
    let result = sqlx::query!("DELETE FROM frogol_avatar_images WHERE frogol_id = ? AND image_filename = ?",
        frogol_id,
        image_filename
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// Deletes all avatar images associated with a frogol from the filesystem and database.
pub async fn delete_all_avatar_images_for_frogol(
    pool: &SqlitePool,
    frogol_id: &str,
    image_save_dir: &PathBuf,
) -> Result<(), AppError> {
    let images_to_delete: Vec<FrogolAvatarImage> = sqlx::query_as!(
        FrogolAvatarImage,
        "SELECT id as \"id!\", frogol_id as \"frogol_id!\", image_filename as \"image_filename!\", created_at as \"created_at!\" FROM frogol_avatar_images WHERE frogol_id = ?",
        frogol_id
    )
    .fetch_all(pool)
    .await?;

    let mut first_error: Option<AppError> = None;

    // Delete image files from filesystem
    for image_record in &images_to_delete {
        if let Err(e) = delete_image_file(&image_record.image_filename, image_save_dir).await {
            tracing::error!(
                "Failed to delete image file {} for frogol {}: {}. Continuing cleanup.",
                image_record.image_filename,
                frogol_id,
                e
            );
            if first_error.is_none() {
                first_error = Some(e);
            }
        }
    }

    // Delete image records from database
    if !images_to_delete.is_empty() {
        match sqlx::query!("DELETE FROM frogol_avatar_images WHERE frogol_id = ?", frogol_id)
            .execute(pool)
            .await
        {
            Ok(_) => tracing::info!("Successfully deleted image DB records for frogol_id: {}", frogol_id),
            Err(e) => {
                tracing::error!("Failed to delete image DB records for frogol {}: {:#}", frogol_id, e);
                if first_error.is_none() {
                    first_error = Some(AppError::Database(e));
                }
            }
        }
    }

    if let Some(err) = first_error {
        Err(err) // Return the first error encountered
    } else {
        Ok(())
    }
}

/// Batch processes multiple images in parallel for better performance
/// Returns a vector of successfully processed images and any errors encountered
pub async fn process_images_batch(
    image_fields: Vec<FieldData<NamedTempFile>>,
    image_save_dir: &PathBuf,
    starting_order: i64,
) -> (Vec<ProcessedImageData>, Vec<AppError>) {
    let mut processed_images = Vec::new();
    let mut errors = Vec::new();
    
    // Create a stream of futures for parallel processing
    let futures = image_fields.into_iter().enumerate().map(|(index, image_field)| {
        let image_save_dir = image_save_dir.clone();
        let image_order = starting_order + index as i64;
        
        async move {
            // Pre-validate before processing
            if image_field.metadata.file_name.is_none() {
                return Err(AppError::ValidationError(format!("Image at index {} has no filename", index)));
            }
            
            let original_file_name = image_field.metadata.file_name.as_ref()
                .map(|name| name.as_str())
                .unwrap_or("unknown");
                
            if original_file_name.is_empty() && image_field.metadata.content_type.is_none() {
                return Err(AppError::ValidationError(format!("Image at index {} has empty filename and no content type", index)));
            }

            // Check file size before processing
            let temp_file_path = image_field.contents.path();
            match fs::metadata(temp_file_path).await {
                Ok(metadata) => {
                    if metadata.len() == 0 {
                        return Err(AppError::ValidationError(format!("Image {} is empty", original_file_name)));
                    }
                }
                Err(e) => {
                    return Err(AppError::ValidationError(format!("Failed to read metadata for {}: {}", original_file_name, e)));
                }
            }

            // Process the image
            process_and_save_image(image_field, &image_save_dir, image_order).await
        }
    });

    // Process up to 4 images concurrently to balance performance and resource usage
    let mut stream = stream::iter(futures).buffer_unordered(4);
    
    while let Some(result) = stream.next().await {
        match result {
            Ok(processed_data) => processed_images.push(processed_data),
            Err(error) => errors.push(error),
        }
    }
    
    (processed_images, errors)
}

/// Saves avatar image metadata to database
pub async fn save_avatar_image_metadata(
    pool: &SqlitePool,
    frogol_id: &str,
    image_data: &ProcessedImageData,
) -> Result<(), AppError> {
    sqlx::query!(
        "INSERT INTO frogol_avatar_images (id, frogol_id, image_filename) VALUES (?, ?, ?)",
        image_data.new_image_id,
        frogol_id,
        image_data.unique_filename
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Gets the current avatar image for a frogol
pub async fn get_frogol_avatar_image(
    pool: &SqlitePool,
    frogol_id: &str,
) -> Result<Option<FrogolAvatarImage>, AppError> {
    let avatar_image = sqlx::query_as!(
        FrogolAvatarImage,
        "SELECT id as \"id!\", frogol_id as \"frogol_id!\", image_filename as \"image_filename!\", created_at as \"created_at!\" FROM frogol_avatar_images WHERE frogol_id = ? ORDER BY created_at DESC LIMIT 1",
        frogol_id
    )
    .fetch_optional(pool)
    .await?;
    
    Ok(avatar_image)
}

/// Batch deletes multiple image files in parallel
pub async fn delete_images_batch(
    image_filenames: Vec<String>,
    image_save_dir: &PathBuf,
) -> Vec<AppError> {
    let futures = image_filenames.into_iter().map(|filename| {
        let image_save_dir = image_save_dir.clone();
        
        async move {
            delete_image_file(&filename, &image_save_dir).await
        }
    });

    // Process deletions in parallel (up to 8 concurrent operations)
    let results: Vec<Result<(), AppError>> = stream::iter(futures)
        .buffer_unordered(8)
        .collect()
        .await;
    
    // Collect any errors
    results.into_iter().filter_map(|result| result.err()).collect()
} 