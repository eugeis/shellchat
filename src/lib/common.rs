use serde::{Deserialize, Serialize};

pub const HEADER_API_KEY: &str = "api-key";

#[derive(Serialize, Deserialize)]
pub struct ShellRequest {
    pub os: String,
    pub shell: String,
    pub prompt: String,
    pub explain: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ShellResponse {
    pub result: String,
    pub error: String,
}
