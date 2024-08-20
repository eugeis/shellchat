use clap::Parser;
use sc_lib::client::{client, ClientCli};

#[tokio::main]
async fn main() {
    client(ClientCli::parse()).await
}
