use std::error::Error;

use axum::async_trait;

use crate::domain::entities::image::GeneratedImage;

#[async_trait]
pub trait ImageGenerationService {
    async fn generate_image_from_prompt(
        &self,
        prompt: &str,
    ) -> Result<GeneratedImage, Box<dyn Error>>;
}
