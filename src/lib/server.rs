use crate::common::{ShellRequest, ShellResponse, HEADER_API_KEY};
use crate::defaults::DEFAULT_API_KEY;
use crate::prompts::Prompts;
use crate::providers::{new_provider, ProviderApi, ProviderConfig};
use crate::tracing::{setup_tracing_console, setup_tracing_file_console};
use actix_web::{web, App, HttpServer, Responder};
use clap::Parser;
use fancy_regex::Regex;
use serde::Deserialize;
use std::fs;
use std::sync::Arc;
use tracing::{error, info};

lazy_static::lazy_static! {
    pub static ref CODE_BLOCK_RE: Regex = Regex::new(r"(?ms)```\w*(.*)```").unwrap();
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub provider: ProviderConfig,
}

impl Config {
    fn from_yaml(file_path: &str) -> Self {
        let absolute_path = fs::canonicalize(file_path)
            .unwrap_or_else(|_| panic!("Failed to resolve the absolute path: {}", file_path));
        let config_content = fs::read_to_string(absolute_path.to_str().unwrap())
            .unwrap_or_else(|_| panic!("Failed to read the configuration file: {:?}", &absolute_path));
        serde_yaml::from_str(&config_content)
            .unwrap_or_else(|_| panic!("Failed to parse the configuration file: {:?}", &absolute_path))
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
            info!(
                "{}/{}: {} => {}",
                &request.os, &request.shell, &&request.prompt, &eval_str
            );
            ShellResponse {
                result: eval_str,
                error: String::new(),
            }
        }
        Err(err) => {
            error!("Error calling provider: {}", err);

            ShellResponse {
                result: String::new(),
                error: err,
            }
        }
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
    #[clap(short = 'u', long, env = "API_URL", default_value = "127.0.0.1:8080")]
    pub url: String,
    #[clap(short = 'k', long, env = "API_KEY")]
    pub key: Option<String>,
    #[clap(short = 'd', long, env = "LOGS_DIR")]
    pub logs_dir: Option<String>,
}

impl ServerCli {
    pub fn config(&self) -> Config {
        Config::from_yaml(&self.config)
    }
}

pub async fn serve(cli: ServerCli) -> std::io::Result<()> {
    match &cli.logs_dir {
        Some(dir) => {
            //we need to keep the guard alive
            let _guard = setup_tracing_file_console(dir, "sc-serve");
        }
        None => {
            setup_tracing_console();
        }
    }

    let config = cli.config();

    let provider = new_provider(&config.provider);

    const INIT_MESSAGE: &str = "hi";
    let data = provider.call("", INIT_MESSAGE).await
        .expect("Provider is not responding");
    info!("{}", format!("{}: {}", INIT_MESSAGE, data));

    let key = Arc::new(
        cli.key
            .clone()
            .unwrap_or_else(|| DEFAULT_API_KEY.to_string()),
    );

    let app_config = Arc::new(AppConfig {
        provider,
        prompts: Prompts::from_yaml_content(include_str!("../../prompts.yaml")),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_config.clone()))
            .app_data(web::Data::new(key.clone()))
            .route("/", web::post().to(chat))
    })
    .bind(&cli.url)?
    .run()
    .await
}
