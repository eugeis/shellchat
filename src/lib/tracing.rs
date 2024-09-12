use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::{fmt, fmt::format::FmtSpan, prelude::*, EnvFilter};

pub fn setup_tracing_file_console(dir: &str, file: &str) -> WorkerGuard {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("INFO"));

    // File logging layer (without color)
    let file_appender = rolling::hourly(dir, file);
    let (file_non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer()
        .with_writer(file_non_blocking)
        .with_ansi(false) // Disable ANSI escape codes for file output
        .with_span_events(FmtSpan::FULL);

    // Console logging layer (with color)
    let console_layer = fmt::layer()
        .with_writer(std::io::stdout) // Or use stderr if you prefer
        .with_ansi(true) // Enable ANSI escape codes for console output
        .with_span_events(FmtSpan::FULL);

    // Combining the layers and setting the global maximum level
    tracing_subscriber::registry()
        .with(file_layer)
        .with(console_layer)
        .with(env_filter) // Set max level for both layers
        .init();

    guard
}

pub fn setup_tracing_console() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("INFO"));

    // Console logging layer
    let console_layer = fmt::layer()
        .with_writer(std::io::stdout) // Output to console (stdout)
        .with_ansi(false) // Disable ANSI escape codes
        .with_span_events(FmtSpan::FULL); // Optional: Include span events

    // Initialize the subscriber with the console layer
    tracing_subscriber::registry()
        .with(console_layer)
        .with(env_filter) // Set log level
        .init();
}
