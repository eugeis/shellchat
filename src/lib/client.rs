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
        let text = self.text.iter().map(|x| x.trim()
            .to_string()).collect::<Vec<String>>().join(" ");
        text
    }
}