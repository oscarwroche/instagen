use std::{env, error::Error, path::Path};

use aws_sdk_s3::{primitives::ByteStream, Client as AwsS3Client};

pub async fn upload_image(file_path: &Path) -> Result<String, Box<dyn Error>> {
    let shared_config = aws_config::load_from_env().await;
    let bucket = S3_IMAGE_BUCKET_NAME;
    let client = AwsS3Client::new(&shared_config);

    let body = ByteStream::from_path(file_path).await?;


    let key = generate_s3_key_with_timestamp();

    client
        .put_object()
        .bucket(bucket)
        .key(&key)
        .body(body)
        .send()
        .await?;

    println!("File uploaded to {}/{}", bucket, key);

    let aws_region = env::var("AWS_REGION")?;

    Ok(format_args!("https://{}.s3.{}.amazonaws.com/{}", bucket, aws_region, key).to_string())
}
