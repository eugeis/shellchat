use actix_web::{web, App, HttpServer, Responder};
use clap::{Arg, Command as ClapCommand};
use shellchat_lib::{InputData, OutputData};

#[derive(Debug)]
enum AiProviderType {
    OpenAI { api_key: String, base_url: String },
    AzureOpenAI { api_key: String, base_url: String },
    Ollama { api_key: String, base_url: String },
}

#[derive(Debug)]
struct AiProvider {
    provider_type: AiProviderType,
}

#[derive(Debug)]
struct Cli {
    address: String,
    port: u16,
    model_name: String,
    ai_provider: AiProvider,
}

impl Cli {
    fn new() -> Self {
        let matches = ClapCommand::new("My Server")
            .arg(Arg::new("address")
                     .long("address")
                     .value_name("ADDRESS")
                     .help("Sets the server address")
                     .env("SERVER_ADDRESS") // Use environment variable
                     .default_value("127.0.0.1") // Fallback default value
            )
            .arg(Arg::new("port")
                     .long("port")
                     .value_name("PORT")
                     .help("Sets the server port")
                     .env("SERVER_PORT") // Use environment variable
                     .default_value("8080") // Fallback default value
            )
            .arg(Arg::new("model")
                     .long("model")
                     .value_name("MODEL")
                     .help("Sets the model name")
                     .env("MODEL_NAME") // Use environment variable
                     .default_value("default_model") // Fallback default value
            )
            .arg(Arg::new("provider")
                     .long("provider")
                     .value_name("PROVIDER")
                     .help("Sets the AI provider type")
                     .env("AI_PROVIDER_TYPE") // Use environment variable
                     .default_value("openai") // Fallback default value
            )
            .arg(Arg::new("api_key")
                     .long("api_key")
                     .value_name("API_KEY")
                     .help("Sets the API key for the AI provider")
                     .env("AI_PROVIDER_API_KEY") // Use environment variable
                     .default_value("default_key") // Fallback default value
            )
            .arg(Arg::new("base_url")
                     .long("base_url")
                     .value_name("BASE_URL")
                     .help("Sets the base URL for the AI provider")
                     .env("AI_PROVIDER_BASE_URL") // Use environment variable
                     .default_value("https://api.example.com") // Fallback default value
            )
            .get_matches();

        let address = matches.get_one::<String>("address").unwrap().clone();
        let port = matches.get_one::<String>("port").unwrap().parse().unwrap_or(8080);
        let model_name = matches.get_one::<String>("model").unwrap().clone();
        let provider_type = matches.get_one::<String>("provider").unwrap().clone();
        let api_key = matches.get_one::<String>("api_key").unwrap().clone();
        let base_url = matches.get_one::<String>("base_url").unwrap().clone();

        let ai_provider = match provider_type.as_str() {
            "openai" => AiProvider {
                provider_type: AiProviderType::OpenAI { api_key, base_url },
            },
            "azure" => AiProvider {
                provider_type: AiProviderType::AzureOpenAI { api_key, base_url },
            },
            "ollama" => AiProvider {
                provider_type: AiProviderType::Ollama { api_key, base_url },
            },
            _ => panic!("Unsupported AI provider type"),
        };

        Self {
            address,
            port,
            model_name,
            ai_provider,
        }
    }
}

async fn execute(data: web::Json<InputData>) -> impl Responder {
    let input_str = &data.data;
    let output_data = OutputData {
        result: format!("Endpoint 1: Received {}", input_str),
    };
    web::Json(output_data)
}

async fn explain(data: web::Json<InputData>) -> impl Responder {
    let input_str = &data.data;
    let output_data = OutputData {
        result: format!("Endpoint 2: Received {}", input_str),
    };
    web::Json(output_data)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::new();

    println!("Server started at {}:{}", cli.address, cli.port);
    println!("Using model: {}", cli.model_name);
    println!("AI Provider: {:?}", cli.ai_provider);

    HttpServer::new(move || {
        App::new()
            .route("/execute", web::post().to(execute))
            .route("/explain", web::post().to(explain))
    })
        .bind(format!("{}:{}", cli.address, cli.port))?
        .run()
        .await
}
