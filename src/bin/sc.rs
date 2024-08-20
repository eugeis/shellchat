use clap::Parser;
use sc_lib::chatter::Chatter;
use sc_lib::client::ClientCli;
use sc_lib::command::IS_STDOUT_TERMINAL;
use sc_lib::defaults::DEFAULT_API_KEY;

#[tokio::main]
async fn main() {
    if !*IS_STDOUT_TERMINAL {
        eprintln!("I can't recognize an terminal");
        return;
    }
    let cli = ClientCli::parse();
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
