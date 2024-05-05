use std::sync::Arc;

use crate::domain::{
    entities::image::{GeneratedImage, SavedImage},
    repositories::image_repository::ImageRepository,
    services::image_generation_service::ImageGenerationService,
};

pub async fn generate_and_upload_image(
    image_generation_service: Arc<dyn ImageGenerationService + Send + Sync>,
    image_repository: Arc<dyn ImageRepository + Send + Sync>,
    prompt: &str,
) -> Result<SavedImage, Box<dyn std::error::Error>> {
    let generated_image: GeneratedImage = image_generation_service
        .generate_image_from_prompt(prompt)
        .await?;

    image_repository.save(&generated_image).await
}
