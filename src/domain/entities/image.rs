use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Image {
    url: String,
}

impl Image {
    pub fn new(url: String) -> Self {
        Image { url }
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}
