use crate::chatter::Chatter;
use crate::command::IS_STDOUT_TERMINAL;
use crate::defaults::DEFAULT_API_KEY;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct ClientCli {
    #[clap(short = 'a', long, env = "ADDRESS", default_value = "http://127.0.0.1")]
    pub address: String,
    #[clap(short = 'p', long, env = "PORT", default_value = "8080")]
    pub port: String,
    #[clap(short = 'k', long, env = "KEY")]
    pub key: Option<String>,
    #[clap(trailing_var_arg = true)]
    text: Vec<String>,
}

impl ClientCli {
    pub fn endpoint(&self) -> String {
        format!("{}:{}", &self.address, &self.port)
    }

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
        .unwrap_or_else(|| DEFAULT_API_KEY.to_string()); // Add this line
    let chatter = Chatter::new(&cli.endpoint(), &api_key);
    let _ = chatter.shell_execute(&text).await;
}
