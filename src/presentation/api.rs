use crate::{
    application::{
        commands::{
            delete_image::delete_image, generate_and_upload_image::generate_and_upload_image,
            post_image::post_image,
        },
        services::auth_service::AuthCredentials,
    },
    infrastructure::{
        repositories::{
            instagram_post_repository::InstagramPostRepository,
            s3_image_repository::S3ImageRepository,
        },
        services::{facebook_auth_service::FacebookAuthService, open_ai_adapter::OpenAIAdapter},
    },
};
use askama::Template;
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::Html,
    routing::{delete, get, get_service, post},
    Router,
};
use axum_server::tls_rustls::bind_rustls;
use dotenv::dotenv;
use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, sync::Arc};
use std::{env, net::SocketAddr};
use tokio::sync::Mutex;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use super::config::load_server_config;

#[derive(Template)]
#[template(path = "img.html")]

struct ImgTemplate<'a> {
    url: &'a str,
    fb_oauth_url: &'a str,
}

#[derive(Clone)]
struct AppState {
    open_ai_adapter: Arc<OpenAIAdapter>,
    instagram_post_repository: Arc<Mutex<InstagramPostRepository>>,
    s3_image_repository: Arc<S3ImageRepository>,
}

pub async fn serve() {
    dotenv().ok();
    let fb_app_id = env::var("FB_APP_ID").unwrap();
    let fb_client_secret = env::var("FB_CLIENT_SECRET").unwrap();
    let ig_user_id = env::var("IG_USER_ID").unwrap();
    let aws_bucket_name = env::var("AWS_BUCKET_NAME").unwrap();
    let aws_region = env::var("AWS_REGION").unwrap();

    let aws_config = aws_config::load_from_env().await;

    let open_ai_adapter = Arc::new(OpenAIAdapter::new());
    let facebook_auth_service = Arc::new(FacebookAuthService::new(
        String::from(fb_app_id),
        String::from(fb_client_secret),
    ));
    let instagram_post_repository = Arc::new(Mutex::new(InstagramPostRepository::new(
        String::from(ig_user_id),
        facebook_auth_service.clone(),
    )));
    let s3_image_repository = Arc::new(S3ImageRepository::new(
        aws_config,
        aws_bucket_name,
        aws_region,
    ));

    let shared_state = AppState {
        open_ai_adapter,
        instagram_post_repository,
        s3_image_repository,
    };

    // ADD HTTPS HERE, see: https://github.com/tokio-rs/axum/blob/main/examples/tls-rustls/src/main.rs

    let api_router = Router::new()
        .route("/images", post(generate_image_from_prompt_handler))
        .route("/images/:id", delete(delete_image_handler))
        .route("/perform_post_action", get(authenticate_and_post_handler))
        .with_state(shared_state);

    tracing_subscriber::fmt::init();

    let app = Router::new()
        .nest("/api", api_router)
        .nest_service("/static", ServeDir::new("static"))
        .fallback(get_service(ServeFile::new("static/index.html")))
        .layer(TraceLayer::new_for_http());

    let config = load_server_config().await;

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Serialize, Deserialize)]
struct GenerateImageFromPrompt {
    prompt: String,
}

async fn generate_image_from_prompt_handler(
    State(state): State<AppState>,
    Json(payload): Json<GenerateImageFromPrompt>,
) -> Html<String> {
    let image = generate_and_upload_image(
        state.open_ai_adapter,
        state.s3_image_repository,
        &(payload.prompt),
    )
    .await
    .unwrap();

    let img = ImgTemplate {
        url: &(image.url()),
        fb_oauth_url: &(generate_fb_oauth_url(image.url())),
    };

    let result = img.render().unwrap();

    Html(result)
}

fn generate_fb_oauth_url(s3_image_url: &str) -> String {
    let fb_app_id = env::var("FB_APP_ID").unwrap();
    let tcp_listener_address = env::var("TCP_LISTENER_ADDRESS").unwrap();
    let redirect_uri = format!("https://{}/api/perform_post_action", tcp_listener_address);
    let state_param = format!(r#"{{"{{s3_file_uri={}}}"}}"#, s3_image_url); // Replace with your actual state parameter

    format!(
        "https://www.facebook.com/v19.0/dialog/oauth?client_id={}&redirect_uri={}&state={}&scope=instagram_basic,instagram_content_publish,pages_show_list",
        fb_app_id, redirect_uri, state_param
    )
}

#[derive(Serialize, Deserialize)]
struct DeleteImage {
    image_id: String,
}

async fn delete_image_handler(
    State(state): State<AppState>,
    Path(image_id): Path<String>,
) -> StatusCode {
    delete_image(state.s3_image_repository, image_id)
        .await
        .unwrap();

    StatusCode::NO_CONTENT
}

async fn authenticate_and_post_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) {
    let code = params.get("code").unwrap();
    let fb_callback_state = params.get("state").unwrap();
    let image_url = extract_image_url_from_state(fb_callback_state)
        .expect("Couldn't extract image url from FB state param");
    let mut auth_credentials = HashMap::new();
    auth_credentials.insert(String::from("code"), code.to_string());

    let instagram_post_repository = state.instagram_post_repository.clone();

    {
        let mut instagram_post_repository_mutex_guard = instagram_post_repository.lock().await;

        let _ = instagram_post_repository_mutex_guard
            .authenticate(AuthCredentials {
                parameters: auth_credentials,
            })
            .await;
    }

    {
        let instagram_post_repository_mutex_guard = instagram_post_repository.lock().await;

        post_image(&*instagram_post_repository_mutex_guard, image_url).await;
    }
}

pub fn extract_image_url_from_state(s: &str) -> Result<String, Box<dyn Error>> {
    let decoded_string = percent_decode_str(s).decode_utf8()?;

    let trimmed = decoded_string.trim_matches(|c| c == '{' || c == '}' || c == '"');

    if !trimmed.starts_with("s3_file_uri=") {
        return Err(format!("Failed to parse S3 File URI").into());
    };
    return Ok(trimmed["s3_file_uri=".len()..].to_string());
}
