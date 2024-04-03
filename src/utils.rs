pub mod utils {
    use chrono::prelude::*;

    pub fn generate_s3_key_with_timestamp() -> String {
        let now = Utc::now();
        // Format the time as you like. Here's an example format: "2023-01-25-15-00-00"
        let time_str = now.format("%Y-%m-%d-%H-%M-%S").to_string();
        format!("generated-picture/{}.jpg", time_str).to_string()
    }
}
