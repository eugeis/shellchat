use crate::common::{Question, HEADER_API_KEY};
use crate::defaults::DEFAULT_API_KEY;
use crate::prompts::Prompts;
use crate::providers::{new_provider, ProviderApi, ProviderConfig};
use crate::tracing::{setup_tracing_console, setup_tracing_file_console};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
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
        let config_content =
            fs::read_to_string(absolute_path.to_str().unwrap()).unwrap_or_else(|_| {
                panic!(
                    "Failed to read the configuration file: {:?}",
                    &absolute_path
                )
            });
        serde_yaml::from_str(&config_content).unwrap_or_else(|_| {
            panic!(
                "Failed to parse the configuration file: {:?}",
                &absolute_path
            )
        })
    }
}

pub struct AppConfig {
    pub provider: Arc<dyn ProviderApi + Send + Sync>,
    pub prompts: Prompts,
}
pub async fn chat(
    request: web::Json<Question>,
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
        return HttpResponse::Unauthorized()
            .body("Client version/build not compatible. Please use corresponding 'shc' client.");
    }

    let prompts = &data.prompts;
    let provider = &data.provider;

    let prompt = if !request.explain {
        &prompts.shell_prompt(&request.os, &request.shell)
    } else {
        &prompts.explain
    };

    match provider.call(prompt, &request.prompt).await {
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
            HttpResponse::Ok().body(eval_str)
        }
        Err(err) => {
            error!("Error calling provider: {:?}", err);
            HttpResponse::InternalServerError().body(format!("Error calling provider: {}", err))
        }
    }
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
    provide_check(&provider).await;

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

async fn provide_check(provider: &Arc<dyn ProviderApi + Send + Sync>) {
    const INIT_MESSAGE: &str = "hi";
    let data = provider.call("", INIT_MESSAGE).await.unwrap_or_else(|err| {
        error!("Provider is not responding: {:?}", err);
        panic!("Provider is not responding: {:?}", err);
    });
    info!("{}", format!("{}: {}", INIT_MESSAGE, data));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::ProviderError;
    use actix_web::{test, web, App};

    const PROMPTS_CONTENT: &str = r#"
        explain: "Explain prompt"
        os_prompt: "Operating system prompt for {os} and {shell}"
        combinator_powershell: "PowerShell combinator"
        combinator_default: "Default combinator"
        additional_instructions: "Additional instructions"
        "#;

    #[actix_web::test]
    async fn test_chat_success() {
        let app_config = Arc::new(AppConfig {
            provider: Arc::new(MockProvider {}),
            prompts: Prompts::from_yaml_content(PROMPTS_CONTENT),
        });

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_config.clone()))
                .app_data(web::Data::new(Arc::new(DEFAULT_API_KEY.to_string())))
                .route("/", web::post().to(chat)),
        )
        .await;

        let question = Question {
            os: "Linux".to_string(),
            shell: "bash".to_string(),
            prompt: "What is Rust?".to_string(),
            explain: false,
        };
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&question)
            .insert_header((HEADER_API_KEY, DEFAULT_API_KEY))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_chat_invalid_api_key() {
        let app_config = Arc::new(AppConfig {
            provider: Arc::new(MockProvider {}),
            prompts: Prompts::from_yaml_content(PROMPTS_CONTENT),
        });

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_config.clone()))
                .app_data(web::Data::new(Arc::new(DEFAULT_API_KEY.to_string())))
                .route("/", web::post().to(chat)),
        )
        .await;

        let question = Question {
            os: "Linux".to_string(),
            shell: "bash".to_string(),
            prompt: "What is Rust?".to_string(),
            explain: false,
        };
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&question)
            .insert_header((HEADER_API_KEY, "invalid_key"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
    }

    struct MockProvider;

    #[async_trait::async_trait]
    impl ProviderApi for MockProvider {
        async fn call(
            &self,
            _role_prompt: &str,
            _user_prompt: &str,
        ) -> Result<String, ProviderError> {
            Ok("Mock response".to_string())
        }
    }

    #[test]
    async fn test_extract_block() {
        let input = "Some text\n```\nCode block\n```";
        let output = extract_block(input);
        assert_eq!(output, "Code block");

        let input_no_block = "Some text without code block";
        let output_no_block = extract_block(input_no_block);
        assert_eq!(output_no_block, input_no_block);
    }

    #[actix_web::test]
    async fn test_chat_with_explain() {
        let app_config = Arc::new(AppConfig {
            provider: Arc::new(MockProvider {}),
            prompts: Prompts::from_yaml_content(PROMPTS_CONTENT),
        });

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_config.clone()))
                .app_data(web::Data::new(Arc::new(DEFAULT_API_KEY.to_string())))
                .route("/", web::post().to(chat)),
        )
        .await;

        let question = Question {
            os: "Linux".to_string(),
            shell: "bash".to_string(),
            prompt: "What is Rust?".to_string(),
            explain: true,
        };
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&question)
            .insert_header((HEADER_API_KEY, DEFAULT_API_KEY))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
