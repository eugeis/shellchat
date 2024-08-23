use crate::chatter::Chatter;
use crate::command::IS_STDOUT_TERMINAL;
use crate::defaults::DEFAULT_API_KEY;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct ClientCli {
    #[clap(
        short = 'u',
        long,
        env = "API_URL",
        default_value = "http://127.0.0.1:8080"
    )]
    pub url: String,
    #[clap(short = 'k', long, env = "API_KEY")]
    pub key: Option<String>,
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
    let chatter = Chatter::new(&cli.url, &api_key);
    match chatter.shell_execute(&text).await {
        Ok(_) => {}
        Err(err) => eprintln!("Error: {}", err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_cli_text() {
        let args = ClientCli {
            url: "http://localhost:8080".to_string(),
            key: None,
            text: vec!["Hello, world!".to_string()],
        };
        assert_eq!(args.text(), "Hello, world!");
    }

    #[tokio::test]
    async fn test_client() {
        let cli = ClientCli {
            url: "http://localhost:8080".to_string(),
            key: Some("test_key".to_string()),
            text: vec!["echo Hello".to_string()],
        };
        client(cli).await;

        let stdout_terminal_state = *IS_STDOUT_TERMINAL;
        assert!(stdout_terminal_state);
    }
}
