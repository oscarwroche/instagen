pub mod application;
pub mod config;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
pub mod utils;

use presentation::api::serve;

#[tokio::main]
async fn main() {
    serve().await;
}
