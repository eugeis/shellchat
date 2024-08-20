use std::fs;
use std::sync::Arc;
use actix_web::{Responder, web};
use clap::Parser;
use fancy_regex::Regex;
use serde::Deserialize;
use tracing::{error, info};
use crate::common::{HEADER_API_KEY, ShellRequest, ShellResponse};
use crate::prompts::Prompts;
use crate::providers::{ProviderApi, ProviderConfig};


lazy_static::lazy_static! {
    pub static ref CODE_BLOCK_RE: Regex = Regex::new(r"(?ms)```\w*(.*)```").unwrap();
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub provider: ProviderConfig,
}

impl Config {
    fn from_yaml(file_path: &str) -> Self {
        let config_content = fs::read_to_string(file_path)
            .unwrap_or_else(|_| panic!("Failed to read the configuration file: {}", file_path));
        serde_yaml::from_str(&config_content).
            unwrap_or_else(|_| panic!("Failed to read the configuration file: {}", file_path))
    }
}

pub struct AppConfig {
    pub provider: Arc<dyn ProviderApi + Send + Sync>,
    pub prompts: Prompts,
}

pub async fn chat(
    request: web::Json<ShellRequest>,
    data: web::Data<Arc<AppConfig>>,
    key: web::Data<Arc<String>>,
    req: actix_web::HttpRequest,
) -> impl Responder {
    let api_key = req
        .headers()
        .get(HEADER_API_KEY)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if api_key != key.as_str() {
        error!("Invalid API key: {}", api_key);
        return actix_web::HttpResponse::Unauthorized().body("Invalid API key");
    }

    let prompts = &data.prompts;
    let provider = &data.provider;

    let prompt = if !request.explain {
        &prompts.shell_prompt(&request.os, &request.shell)
    } else {
        &prompts.explain
    };

    let response = match provider.call(prompt, &request.prompt).await {
        Ok(mut eval_str) => {
            if !request.explain {
                if let Ok(true) = CODE_BLOCK_RE.is_match(&eval_str) {
                    eval_str = extract_block(&eval_str);
                }
            }
            info!("{}/{}: {} => {}", &request.os, &request.shell, &&request.prompt, &eval_str);
            ShellResponse {
                result: eval_str,
                error: String::new(),
            }
        },
        Err(err) => {
            error!("Error calling provider: {}", err);

            ShellResponse {
                result: String::new(),
                error: err,
            }
        },
    };

    actix_web::HttpResponse::Ok().json(response)
}

pub fn extract_block(input: &str) -> String {
    let output: String = CODE_BLOCK_RE
        .captures_iter(input)
        .filter_map(|m| {
            m.ok()
                .and_then(|cap| cap.get(1))
                .map(|m| String::from(m.as_str()))
        })
        .collect();
    if output.is_empty() {
        input.trim().to_string()
    } else {
        output.trim().to_string()
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct ServerCli {
    #[clap(short = 'c', long, env = "CONFIG", default_value = "config.yaml")]
    pub config: String,
    #[clap(short = 'a', long, env = "ADDRESS", default_value = "127.0.0.1")]
    pub address: String,
    #[clap(short = 'p', long, env = "PORT", default_value = "8080")]
    pub port: String,
    #[clap(short = 'k', long, env = "KEY")]
    pub key: Option<String>,
    #[clap(short = 'd', long, env = "LOGS_DIR")]
    pub logs_dir: Option<String>,
}

impl ServerCli {
    pub fn config(&self) -> Config {
        Config::from_yaml(&self.config)
    }

    pub fn endpoint(&self) -> String {
        format!("{}:{}", &self.address, &self.port)
    }
}