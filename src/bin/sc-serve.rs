use actix_web::{web, App, HttpServer};
use clap::Parser;
use sc_lib::defaults::DEFAULT_API_KEY;
use sc_lib::prompts::Prompts;
use sc_lib::providers::new_provider;
use sc_lib::server::{chat, AppConfig, ServerCli};
use sc_lib::tracing::{setup_tracing_console, setup_tracing_file_console};
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cli = ServerCli::parse();

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
    let key = Arc::new(
        cli.key
            .clone()
            .unwrap_or_else(|| DEFAULT_API_KEY.to_string()),
    );

    let app_config = Arc::new(AppConfig {
        provider: new_provider(&config.provider),
        prompts: Prompts::from_yaml_content(include_str!("../../prompts.yaml")),
    });

    let endpoint = &cli.endpoint();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_config.clone()))
            .app_data(web::Data::new(key.clone()))
            .route("/", web::post().to(chat))
    })
    .bind(endpoint)?
    .run()
    .await
}
