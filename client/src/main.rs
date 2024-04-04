fn main() {
    println!("Client");
}

pub const FB_APP_ID: &str = "986335749574127";
pub const FB_OAUTH_REDIRECT_URL: &str = "https://127.0.0.1:8080";

async fn open_fb_oauth_url(s3_file_uri: String) -> Result<(), Box<dyn Error>> {
    let app_id = FB_APP_ID;
    let redirect_uri = FB_OAUTH_REDIRECT_URL;
    let state_param = format!(r#"{{"{{s3_file_uri={}}}"}}"#, s3_file_uri); // Replace with your actual state parameter

    let url = format!(
        "https://www.facebook.com/v19.0/dialog/oauth?client_id={}&redirect_uri={}&state={}&scope=instagram_basic,instagram_content_publish,pages_show_list",
        app_id, redirect_uri, state_param
    );

    webbrowser::open(&url)?;

    Ok(())
}
