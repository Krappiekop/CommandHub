mod app_state;
mod config;
mod controllers;
mod execution;
mod models;
mod registry;
mod routes;

use std::error::Error;
use std::sync::Arc;

#[tokio::main]
// De hoofdfunctie van de applicatie
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialiseer de tracing-subscriber voor logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Laad de configuratie en maak een ToolRegistry
    let config = config::load()?;
    let aantal_tools = config.tools.len();
    let registry = Arc::new(registry::ToolRegistry::new(config));

    let state = app_state::AppState {
        registry,
        execution: Arc::new(execution::ExecutionService::new()),
    };

    tracing::info!(aantal_tools, "Configuratie geladen");

    // Start de TCP-server en luister op poort 3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    tracing::info!(adres = %listener.local_addr()?, "CommandHub gestart");

    // Start de Axum-server met de router en registreer een handler voor Ctrl+C
    let execution = Arc::clone(&state.execution);

    let server_result = axum::serve(listener, routes::router(state.clone()))
        .with_graceful_shutdown(async move {
            if let Err(fout) = tokio::signal::ctrl_c().await {
                tracing::error!(error = %fout, "Ctrl+C afhandeling mislukt");
            }

            execution.begin_shutdown();
        })
        .await;

    state.execution.shutdown().await;
    server_result?;

    Ok(())
}
