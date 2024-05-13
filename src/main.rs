pub mod application;
pub mod config;
pub mod domain;
pub mod infrastructure;
pub mod presentation;

use presentation::api::serve;

#[tokio::main]
async fn main() {
    serve().await;
}
