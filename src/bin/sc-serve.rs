use clap::Parser;
use sc_lib::server::{serve, ServerCli};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    serve(ServerCli::parse()).await
}
