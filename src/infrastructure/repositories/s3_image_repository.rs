use std::error::Error;

use aws_config::SdkConfig;
use aws_sdk_s3::{primitives::ByteStream, Client as S3Client};
use axum::async_trait;
use serde_json::Value;

use crate::domain::{
    entities::image::{GeneratedImage, SavedImage},
    repositories::image_repository::ImageRepository,
};

pub struct S3ImageRepository {
    s3_client: S3Client,
    bucket_name: String,
    region: String,
}

impl S3ImageRepository {
    pub fn new(aws_config: SdkConfig, bucket_name: String, region: String) -> S3ImageRepository {
        let s3_client = S3Client::new(&aws_config);

        S3ImageRepository {
            s3_client,
            bucket_name,
            region,
        }
    }
}

#[async_trait]
impl ImageRepository for S3ImageRepository {
    async fn save(&self, image: &GeneratedImage) -> Result<SavedImage, Box<dyn Error>> {
        // Decode the base64 string into bytes
        let image_bytes = base64::decode(image.data())?;

        // Convert the bytes into a ByteStream for the S3 upload
        let byte_stream = ByteStream::from(image_bytes);

        self.s3_client
            .put_object()
            .bucket(&self.bucket_name)
            .key(image.id()) // Using URL as key, adjust if necessary
            .body(byte_stream)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        // Construct the URL of the uploaded image
        let url = format!(
            "https://{}.s3.{}.amazonaws.com/{}",
            self.bucket_name,
            self.region,
            image.id()
        );

        Ok(SavedImage::new(String::from(image.id()), url))
    }
}
