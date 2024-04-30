use std::sync::Arc;

use crate::domain::{
    entities::image::Image, services::image_generation_service::ImageGenerationService,
};

pub async fn generate_image_from_prompt(
    image_generation_service: Arc<dyn ImageGenerationService + Send + Sync>,
    prompt: &str,
) -> Result<Image, Box<dyn std::error::Error>> {
    image_generation_service
        .generate_image_from_prompt(prompt)
        .await
}
