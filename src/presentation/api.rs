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
    extract::{Json, Query, State},
    http::StatusCode,
    response::Html,
    routing::{delete, post},
    Router,
};
use dotenv::dotenv;
use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use std::env;
use std::{collections::HashMap, error::Error, sync::Arc};
use tokio::{spawn, sync::Mutex};
use tower_http::services::ServeFile;
use tower_http::trace::TraceLayer;

#[derive(Template)]
#[template(path = "img.html")]

struct ImgTemplate<'a> {
    url: &'a str,
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
    let tcp_listener_address = env::var("TCP_LISTENER_ADDRESS").unwrap();
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

    let api_router = Router::new()
        .route("/images", post(generate_image_from_prompt_handler))
        .route("/images", delete(delete_image_handler))
        .route("/posts", post(authenticate_and_post_handler))
        .with_state(shared_state)
        .layer(TraceLayer::new_for_http());

    tracing_subscriber::fmt::init();

    let app = Router::new()
        .nest("/api", api_router)
        .nest_service("/", ServeFile::new("static/index.html"));

    let listener = tokio::net::TcpListener::bind(tcp_listener_address)
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
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
    };

    let result = img.render().unwrap();

    Html(result)
}

#[derive(Serialize, Deserialize)]
struct DeleteImage {
    image_id: String,
}

async fn delete_image_handler(
    State(state): State<AppState>,
    Json(payload): Json<DeleteImage>,
) -> StatusCode {
    delete_image(state.s3_image_repository, payload.image_id)
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

    spawn(async move {
        let mut instagram_post_repository_mutex_guard = instagram_post_repository.lock().await;

        let _ = instagram_post_repository_mutex_guard
            .authenticate(AuthCredentials {
                parameters: auth_credentials,
            })
            .await;
    });

    let instagram_post_repository = state.instagram_post_repository.clone();

    spawn(async move {
        let instagram_post_repository_mutex_guard = instagram_post_repository.lock().await;

        post_image(&*instagram_post_repository_mutex_guard, image_url).await;
    });
}

pub fn extract_image_url_from_state(s: &str) -> Result<String, Box<dyn Error>> {
    let decoded_string = percent_decode_str(s).decode_utf8()?;

    println!("Decoded string: {}", decoded_string);

    let trimmed = decoded_string.trim_matches(|c| c == '{' || c == '}' || c == '"');

    if !trimmed.starts_with("file_uri=") {
        return Err(format!("Failed to parse S3 File URI").into());
    };
    return Ok(trimmed["s3_file_uri=".len()..].to_string());
}
