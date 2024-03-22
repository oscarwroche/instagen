use async_openai::{
    types::{CreateImageRequestArgs, ImageModel, ImageSize, ResponseFormat},
    Client,
};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // create client, reads OPENAI_API_KEY environment variable for API key.
    let client = Client::new();

    let request = CreateImageRequestArgs::default()
	.model(ImageModel::DallE3)
        .prompt("Generate a comical manga image featuring an extremely muscular character resembling guts from berserk, he is smoking a cigar while drinking tea in deep meditation. he is wielding a colossal greatsword")
        .n(1)
        .response_format(ResponseFormat::Url)
        .size(ImageSize::S1024x1024)
        .user("async-openai")
        .build()?;

    let response = client.images().create(request).await?;

    // Download and save images to ./data directory.
    // Each url is downloaded and saved in dedicated Tokio task.
    // Directory is created if it doesn't exist.
    let paths = response.save("./data").await?;

    paths
        .iter()
        .for_each(|path| println!("Image file path: {}", path.display()));

    Ok(())
}
