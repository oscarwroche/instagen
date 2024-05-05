use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GeneratedImage {
    id: String,
    data: String,
}

impl GeneratedImage {
    pub fn new(id: String, data: String) -> Self {
        Self { id, data }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data(&self) -> &str {
        &self.data
    }
}

#[derive(Serialize, Deserialize)]
pub struct SavedImage {
    id: String,
    url: String,
}

impl SavedImage {
    pub fn new(id: String, url: String) -> Self {
        Self { id, url }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}

pub enum Image {
    Generated(GeneratedImage),
    Saved(SavedImage),
}
