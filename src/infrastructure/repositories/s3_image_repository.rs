use aws_config::SdkConfig;
use aws_sdk_s3::{Client as S3Client, Config, Region};
use aws_types::Credentials;

pub struct S3ImageRepository {
    s3_client: S3Client,
    bucket_name: String,
}

impl S3ImageRepository {
    fn new(aws_config: SdkConfig, bucket_name: String) -> S3ImageRepository {
        let s3_client = S3Client::new(&shared_config);

        S3ImageRepository {
            s3_client,
            bucket_name,
        }
    }
}

impl ImageRepository for S3ImageRepository {
    async fn save(&self, image: &Image) -> Result<(), Box<dyn Error>> {
        let content = ByteStream::from_path(image.url())
            .await
            .map_err(|e| e.to_string())?;
        self.s3_client
            .put_object()
            .bucket(&self.bucket)
            .key(image.url()) // Using URL as key, adjust if necessary
            .body(content)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}
