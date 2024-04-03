use async_openai::{
    types::{CreateImageRequestArgs, ImageModel, ImageSize, ResponseFormat},
    Client,
};
use server::server::spawn_server;
use std::fs; // For file deletion
use std::io; // Adjusted io import for writing to stdout immediately
use std::path::Path; // For path operations
use std::{env, error::Error};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use webbrowser;

pub mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let is_debug = args.get(3);

    // add channel here and pass it to server. await receiving message after opening page

    let (tx, mut rx): (Sender<String>, Receiver<String>) = channel(1);

    let server_handle = spawn_server(tx).await;

    match is_debug {
        Some(_) => println!("Debug mode: image generation skipped"),
        None => generate_image_from_prompt().await?,
    }

    let app_id = "986335749574127"; // Replace with your actual app ID
    let redirect_uri = "https://127.0.0.1:8080"; // Replace with your actual redirect URI
    let state_param = "abc"; // Replace with your actual state parameter

    let url = format!(
        "https://www.facebook.com/v19.0/dialog/oauth?client_id={}&redirect_uri={}&state={}&scope=instagram_basic,instagram_content_publish,pages_show_list",
        app_id, redirect_uri, state_param
    );

    if webbrowser::open(&url).is_ok() {
        println!("Opened {} in the default web browser.", url);
    } else {
        println!("Failed to open URL.");
    }

    // Here we wait to receive the redirect URI from the server
    match rx.recv().await {
        Some(_) => println!("Image upload succeeded"),
        None => println!("Channel was closed"),
    }

    // Wait for the server task to complete (if needed)
    let _ = server_handle.await;

    Ok(())
}

async fn generate_image_from_prompt() -> Result<(), Box<dyn Error>> {
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
                return Ok(());
            } else {
                // Delete the image if the user is not satisfied
                fs::remove_file(Path::new(first_path))?;
                println!("Image deleted. Let's try again.");
            }
        } else {
            println!("No image was generated. Please try again.");
        }
    }
}
