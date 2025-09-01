use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrogolAvatarImage {
    pub id: String,
    pub frogol_id: String,
    pub image_filename: String,
    pub created_at: String,
}
