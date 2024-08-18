use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct InputData {
    pub data: String,
}

#[derive(Serialize, Deserialize)]
pub struct OutputData {
    pub result: String,
}