use serde::{Deserialize, Serialize};

pub const HEADER_API_KEY: &str = "api-key";

#[derive(Serialize, Deserialize)]
pub struct Question {
    pub os: String,
    pub shell: String,
    pub prompt: String,
    pub explain: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Error {
    pub message: String,
    pub code: Option<u16>, // You can include an error code if applicable
}
