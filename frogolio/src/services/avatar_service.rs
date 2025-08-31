use std::path::PathBuf;
use sqlx::SqlitePool;
use axum_typed_multipart::FieldData;
use tempfile::NamedTempFile;

use crate::errors::AppError;
use crate::handler::image_handler::{
    process_and_save_image,
    save_avatar_image_metadata,
    get_frogol_avatar_image,
    delete_all_avatar_images_for_frogol,
};

pub struct AvatarService {
    pool: SqlitePool,
    image_save_dir: PathBuf,
}

impl AvatarService {
    pub fn new(pool: SqlitePool, image_save_dir: PathBuf) -> Self {
        Self {
            pool,
            image_save_dir,
        }
    }

    /// Uploads a new avatar image for a frogol
    pub async fn upload_avatar(
        &self,
        frogol_id: &str,
        image_field: FieldData<NamedTempFile>,
    ) -> Result<String, AppError> {
        // Delete any existing avatar images for this frogol
        self.delete_avatar(frogol_id).await?;

        // Process and save the new image
        let processed_data = process_and_save_image(
            image_field,
            &self.image_save_dir,
            0, // Only one avatar per frogol
        ).await?;

        // Save metadata to database
        save_avatar_image_metadata(&self.pool, frogol_id, &processed_data).await?;

        Ok(processed_data.unique_filename)
    }

    /// Gets the current avatar image filename for a frogol
    pub async fn get_avatar_filename(&self, frogol_id: &str) -> Result<Option<String>, AppError> {
        let avatar_image = get_frogol_avatar_image(&self.pool, frogol_id).await?;
        Ok(avatar_image.map(|img| img.image_filename))
    }

    /// Deletes the avatar image for a frogol
    pub async fn delete_avatar(&self, frogol_id: &str) -> Result<(), AppError> {
        delete_all_avatar_images_for_frogol(&self.pool, frogol_id, &self.image_save_dir).await
    }

    /// Gets the full URL for an avatar image
    pub fn get_avatar_url(&self, filename: &str) -> String {
        format!("/static/avatars/{}", filename)
    }
}
