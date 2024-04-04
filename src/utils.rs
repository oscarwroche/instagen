use chrono::prelude::*;
use percent_encoding::percent_decode_str;
use std::error::Error;

pub fn generate_s3_key_with_timestamp() -> String {
    let now = Utc::now();
    // Format the time as you like. Here's an example format: "2023-01-25-15-00-00"
    let time_str = now.format("%Y-%m-%d-%H-%M-%S").to_string();
    format!("{}.jpg", time_str).to_string()
}

// Helper function to extract query parameter values
pub fn extract_query_parameter<'a>(request: &'a str, param_name: &str) -> Option<&'a str> {
    if let Some(start_pos) = request.find(&format!("{}=", param_name)) {
        let value_start = start_pos + param_name.len() + 1;
        let value_end = request[value_start..]
            .find('&')
            .map_or_else(|| request[value_start..].find(' ').unwrap_or(0), |v| v)
            + value_start;

        return Some(&request[value_start..value_end]);
    }
    None
}

pub fn extract_s3_file_uri_from_state(s: String) -> Result<String, Box<dyn Error>> {
    // Trim leading and trailing characters

    let decoded_string = percent_decode_str(&s).decode_utf8()?;

    println!("Decoded string: {}", decoded_string);

    let trimmed = decoded_string.trim_matches(|c| c == '{' || c == '}' || c == '"');

    // Check if the string starts with "st="
    if !trimmed.starts_with("s3_file_uri=") {
        return Err(format!("Failed to parse S3 File URI").into());
        // Return the part after "st="
    };
    return Ok(trimmed["s3_file_uri=".len()..].to_string());
}
