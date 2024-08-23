use clap::Parser;
use shc_lib::server::{serve, ServerCli};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    serve(ServerCli::parse()).await
}
