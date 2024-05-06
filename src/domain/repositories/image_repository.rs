use crate::domain::entities::image::{GeneratedImage, SavedImage};
use axum::async_trait;
use std::error::Error;

#[async_trait]
pub trait ImageRepository {
    async fn save(&self, image: &GeneratedImage) -> Result<SavedImage, Box<dyn Error>>;
    async fn delete(&self, image_id: String) -> Result<(), Box<dyn Error>>;
}
