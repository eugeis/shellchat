use std::fs;
use std::sync::Arc;
use actix_web::{App, HttpServer, Responder, web};
use clap::Parser;
use serde::Deserialize;
use fancy_regex::Regex;
use shellchat_lib::common::{HEADER_API_KEY, ShellRequest, ShellResponse};
use shellchat_lib::prompts::Prompts;
use shellchat_lib::providers::{new_provider, ProviderApi, ProviderConfig};
use shellchat_lib::defaults::DEFAULT_API_KEY;

lazy_static::lazy_static! {
    pub static ref CODE_BLOCK_RE: Regex = Regex::new(r"(?ms)```\w*(.*)```").unwrap();
}

#[derive(Debug, Deserialize, Clone)]
struct Config {
    provider: ProviderConfig,
}

impl Config {
    fn from_yaml(file_path: &str) -> Self {
        let config_content = fs::read_to_string(file_path).expect("Failed to read the configuration file");
        serde_yaml::from_str(&config_content).expect("Failed to parse the configuration file")
    }
}

struct AppConfig {
    provider: Arc<dyn ProviderApi + Send + Sync>,
    prompts: Prompts,
}

async fn chat(
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
        return actix_web::HttpResponse::Unauthorized().body("Invalid API key");
    }

    let prompts = &data.prompts;
    let provider = &data.provider;

    let prompt = if !request.explain {
        &prompts.shell_prompt(&request.os, &request.shell)
    } else {
        &prompts.explain
    };

    let response = match provider.call(&prompt, &request.prompt).await {
        Ok(mut eval_str) => {
            if !request.explain {
                if let Ok(true) = CODE_BLOCK_RE.is_match(&eval_str) {
                    eval_str = extract_block(&eval_str);
                }
            }

            ShellResponse {
                result: eval_str,
                error: String::new(),
            }
        },
        Err(error) => ShellResponse {
            result: String::new(),
            error,
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
pub struct Cli {
    #[clap(short = 'c', long, env = "CONFIG", default_value = "config.yaml")]
    pub config: String,
    #[clap(short = 'a', long, env = "ADDRESS", default_value = "127.0.0.1")]
    pub address: String,
    #[clap(short = 'p', long, env = "PORT", default_value = "8080")]
    pub port: String,
    #[clap(short = 'k', long, env = "KEY")]
    pub key: Option<String>,
}

impl Cli {
    pub fn config(&self) -> Config {
        Config::from_yaml(&self.config)
    }

    pub fn endpoint(&self) -> String {
        format!("{}:{}", &self.address, &self.port)
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    let config = cli.config();
    let key = Arc::new(cli.key.clone().unwrap_or_else(||  DEFAULT_API_KEY.to_string()));

    let app_config = Arc::new(AppConfig {
        provider: new_provider(&config.provider),
        prompts: Prompts::from_yaml_content(include_str!("../../prompts.yaml")),
    });

    let endpoint = &cli.endpoint();
    println!("server started at {}", &endpoint);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_config.clone()))
            .app_data(web::Data::new(key.clone()))
            .route("/", web::post().to(chat))
    })
        .bind(endpoint)?
        .run()
        .await
}