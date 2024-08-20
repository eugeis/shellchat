use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::panic::panic_any;
use std::sync::Arc;

#[async_trait]
pub trait ProviderApi {
    async fn call(&self, role_prompt: &str, user_prompt: &str) -> Result<String, String>;
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ProviderConfig {
    OpenAI {
        api_key: String,
        base_url: String,
        model: String,
    },
    AzureOpenAI {
        api_key: String,
        base_url: String,
        model: String,
    },
    Ollama {
        api_key: String,
        base_url: String,
        model: String,
    },
}

#[derive(Clone)]
pub struct AzureOpenAI {
    pub client: Client,
    pub model: String,
    pub api_key: String,
    pub url_full: String,
}

#[derive(serde::Deserialize)]
pub struct CompletionResponse {
    choices: Vec<Choice>,
}

#[derive(serde::Deserialize)]
pub struct Choice {
    message: Message,
}

#[derive(serde::Deserialize)]
pub struct Message {
    content: String,
}

#[async_trait]
impl ProviderApi for AzureOpenAI {
    async fn call(&self, role_prompt: &str, user_prompt: &str) -> Result<String, String> {
        let messages = vec![
            json!({ "role": "system", "content": role_prompt }),
            json!({ "role": "user", "content": user_prompt }),
        ];

        let body = json!({
            "model": &self.model,
            "messages": messages,
        });

        let response = self
            .client
            .post(&self.url_full)
            .header("api-key", &self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;

        let resp: CompletionResponse =
            serde_json::from_str(&response).map_err(|e| e.to_string())?;

        if let Some(choice) = resp.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Ok("".to_string())
        }
    }
}

pub fn new_provider(provider_type: &ProviderConfig) -> Arc<dyn ProviderApi + Send + Sync> {
    let client = Client::new();
    let provider: Arc<dyn ProviderApi + Send + Sync> = match provider_type {
        ProviderConfig::AzureOpenAI {
            api_key,
            base_url,
            model,
        } => Arc::new(AzureOpenAI {
            client,
            model: model.clone(),
            api_key: api_key.clone(),
            url_full: format!(
                "{}/openai/deployments/{}/chat/completions?api-version=2024-02-01",
                &base_url, &model,
            ),
        }),
        _ => panic_any(format!(
            "the provider not implemented yet: {:?}",
            provider_type
        )),
    };
    provider
}
