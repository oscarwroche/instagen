pub struct Post {
    pub url: String,
}

impl Post {
    fn new(id: String, url: String, prompt: String) -> Post {
        Post { url }
    }
}
