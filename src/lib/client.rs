use crate::chatter::Chatter;
use crate::command;
use crate::command::IS_STDOUT_TERMINAL;
use crate::defaults::DEFAULT_API_KEY;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct ClientCli {
    #[clap(
        short = 'u',
        long,
        env = "SHC_API_URL",
        default_value = "http://127.0.0.1:8080"
    )]
    pub url: String,
    #[clap(short = 'k', long, env = "SHC_API_KEY")]
    pub key: Option<String>,
    #[clap(short = 'o', long, env = "SHC_ОS")]
    pub os: Option<String>,
    #[clap(short = 's', long, env = "SHC_SHELL")]
    pub shell: Option<String>,
    #[clap(short = 'e', long)]
    pub explain: bool,
    #[clap(trailing_var_arg = true)]
    pub text: Vec<String>,
}

impl ClientCli {
    pub fn text(&self) -> String {
        let text = self
            .text
            .iter()
            .map(|x| x.trim().to_string())
            .collect::<Vec<String>>()
            .join(" ");
        text
    }
}

pub async fn client(cli: ClientCli) {
    if !*IS_STDOUT_TERMINAL {
        eprintln!("I can't recognize an terminal");
        return;
    }

    let text = cli.text();
    if text.is_empty() {
        eprintln!("How can I assist you in your shell?");
        return;
    };

    let api_key = cli
        .key
        .clone()
        .unwrap_or_else(|| DEFAULT_API_KEY.to_string());

    let os = cli.os.unwrap_or_else(|| command::OS.clone());

    let shell = cli.shell.unwrap_or_else(|| command::SHELL.name.clone());

    let chatter = Chatter::new(&cli.url, &api_key, &os, &shell);

    if cli.explain {
        match chatter.chat(&text, true).await {
            Ok(response) => {
                termimad::print_text(&response);
            }
            Err(err) => {
                eprintln!("Error: {}", err);
            }
        }
    } else {
        match chatter.execute(&text).await {
            Ok(_) => {}
            Err(err) => eprintln!("Error: {}", err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{Question, HEADER_API_KEY};
    use crate::prompts::Prompts;
    use crate::providers::{ProviderApi, ProviderError};
    use crate::server::{chat, AppConfig, Config};
    use actix_web::{test, web, App};
    use std::sync::Arc;

    const PROMPTS_CONTENT: &str = r#"
        explain: "Explain prompt"
        os_prompt: "Operating system prompt for {os} and {shell}"
        combinator_powershell: "PowerShell combinator"
        combinator_default: "Default combinator"
        additional_instructions: "Additional instructions"
        "#;

    #[test]
    async fn test_client_cli_text() {
        let args = ClientCli {
            url: "http://localhost:8080".to_string(),
            key: None,
            os: None,
            shell: None,
            explain: false,
            text: vec!["Hello, world!".to_string()],
        };
        assert_eq!(args.text(), "Hello, world!");
    }

    #[tokio::test]
    async fn test_client() {
        let cli = ClientCli {
            url: "http://localhost:8080".to_string(),
            key: Some("test_key".to_string()),
            os: None,
            shell: None,
            explain: false,
            text: vec!["echo Hello".to_string()],
        };
        client(cli).await;
    }

    #[test]
    async fn test_client_cli_text_empty() {
        let args = ClientCli {
            url: "http://localhost:8080".to_string(),
            key: None,
            os: None,
            shell: None,
            explain: false,
            text: vec![],
        };
        assert_eq!(args.text(), "");
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

    #[actix_web::test]
    async fn test_chat_invalid_body() {
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

        let req = test::TestRequest::post()
            .uri("/")
            .set_payload("invalid_body")
            .insert_header((HEADER_API_KEY, DEFAULT_API_KEY))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
    }

    #[test]
    async fn test_config_from_yaml_file_not_found() {
        let result = std::panic::catch_unwind(|| Config::from_yaml("non_existent_file.yaml"));
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_chat_with_error_response() {
        let app_config = Arc::new(AppConfig {
            provider: Arc::new(MockErrorProvider {}),
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
        assert!(resp.status().is_server_error());
    }

    struct MockErrorProvider;

    #[async_trait::async_trait]
    impl ProviderApi for MockErrorProvider {
        async fn call(
            &self,
            _role_prompt: &str,
            _user_prompt: &str,
        ) -> Result<String, ProviderError> {
            Err(ProviderError::UnexpectedResponse("error".to_string()))
        }
    }
}
