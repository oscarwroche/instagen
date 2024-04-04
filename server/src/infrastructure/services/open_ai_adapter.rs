use std::{env, error::Error};

use crate::domain::{
    entities::image::Image, services::image_generation_service::ImageGenerationService,
};
use async_openai::{
    config::OpenAIConfig,
    types::{CreateImageRequestArgs, ImageModel, ImageSize, ResponseFormat},
    Client,
};
use axum::async_trait;

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
    async fn generate_image_from_prompt(&self, prompt: &str) -> Result<Image, Box<dyn Error>> {
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
            async_openai::types::Image::Url { url: image_url, .. } => {
                Ok(Image::new(image_url.to_string())) // Assuming Image::new accepts a &str or similar
            }
            _ => Err("Expected URL image type but got another type.".into()),
        }
    }
}
