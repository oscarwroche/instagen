use async_openai::{
    types::{CreateImageRequestArgs, ImageModel, ImageSize, ResponseFormat},
    Client,
};
use std::error::Error;
use std::fs; // For file deletion
use std::io::{self}; // Adjusted io import for writing to stdout immediately
use std::path::Path; // For path operations

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // create client, reads OPENAI_API_KEY environment variable for API key.
    let client = Client::new();

    loop {
        // Prompt the user for input
        println!("Enter your prompt for image generation:");
        let mut user_prompt = String::new();
        io::stdin().read_line(&mut user_prompt)?;
        user_prompt = user_prompt.trim().to_string(); // Clean up the input

        let request = CreateImageRequestArgs::default()
            .model(ImageModel::DallE3)
            .prompt(&user_prompt)
            .n(1)
            .response_format(ResponseFormat::Url)
            .size(ImageSize::S1024x1024)
            .user("async-openai")
            .build()?;

        let response = client.images().create(request).await?;

        // Assuming response.save("./data") returns a Vec<String> of file paths
        let paths = response.save("./data").await?;
        
        // Open the first image for the user to review
        if let Some(first_path) = paths.get(0) {
            open::that(&first_path)?; // Use the `open` crate to open the image file
            println!("Image saved to: {}", first_path.display());
            println!("Is this image correct? (y/n):");

            let mut confirmation = String::new();
            io::stdin().read_line(&mut confirmation)?;
            if confirmation.trim().eq_ignore_ascii_case("y") {
                break; // Exit the loop if the user is satisfied
            } else {
                // Delete the image if the user is not satisfied
                fs::remove_file(Path::new(first_path))?;
                println!("Image deleted. Let's try again.");
            }
        } else {
            println!("No image was generated. Please try again.");
        }
    }

    Ok(())
}
