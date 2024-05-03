use crate::domain::entities::image::Image;
use axum::async_trait;
use std::error::Error;

#[async_trait]
pub trait ImageRepository {
    async fn save(&self, image: &Image) -> Result<(), Box<dyn Error>>; // Error handling simplified
}
