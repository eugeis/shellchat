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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_question_serialization() {
        let question = Question {
            os: "Linux".to_string(),
            shell: "bash".to_string(),
            prompt: "What is Rust?".to_string(),
            explain: false,
        };

        let json = serde_json::to_string(&question).unwrap();
        let deserialized_question: Question = serde_json::from_str(&json).unwrap();

        assert_eq!(question.os, deserialized_question.os);
        assert_eq!(question.shell, deserialized_question.shell);
        assert_eq!(question.prompt, deserialized_question.prompt);
        assert_eq!(question.explain, deserialized_question.explain);
    }
}
