use std::{env, error::Error};

use crate::domain::{
    entities::image::GeneratedImage, services::image_generation_service::ImageGenerationService,
};
use async_openai::{
    config::OpenAIConfig,
    types::{CreateImageRequestArgs, ImageModel, ImageSize, ResponseFormat},
    Client,
};
use axum::async_trait;
use uuid::Uuid;

pub struct OpenAIAdapter {
    client: Client<OpenAIConfig>,
}

impl OpenAIAdapter {
    pub fn new() -> Self {
        let client = Client::<OpenAIConfig>::new();
        Self { client }
    }
}

#[async_trait]
impl ImageGenerationService for OpenAIAdapter {
    async fn generate_image_from_prompt(
        &self,
        prompt: &str,
    ) -> Result<GeneratedImage, Box<dyn Error>> {
        let openai_user_name = env::var("OPENAI_USER_NAME").unwrap();

        let request = CreateImageRequestArgs::default()
            .model(ImageModel::DallE3)
            .prompt(prompt)
            .n(1)
            .response_format(ResponseFormat::Url)
            .size(ImageSize::S1024x1024)
            .user(openai_user_name)
            .build()?;

        let response = &self.client.images().create(request).await?;

        match &*response.data[0] {
            async_openai::types::Image::B64Json { b64_json, .. } => Ok(GeneratedImage::new(
                Uuid::new_v4().to_string(),
                b64_json.to_string(),
            )),
            _ => Err("Expected URL image type but got another type.".into()),
        }
    }
}
