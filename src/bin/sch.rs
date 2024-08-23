use clap::Parser;
use shc_lib::client::{client, ClientCli};

#[tokio::main]
async fn main() {
    client(ClientCli::parse()).await
}
