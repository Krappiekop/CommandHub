mod config;
mod controllers;
mod models;
mod routes;

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = config::load()?;
    tracing::info!(aantal_tools = config.tools.len(), "Configuratie geladen");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    tracing::info!(adres = %listener.local_addr()?, "CommandHub gestart");

    axum::serve(listener, routes::router())
        .with_graceful_shutdown(async {
            if let Err(fout) = tokio::signal::ctrl_c().await {
                tracing::error!(error = %fout, "Ctrl+C afhandeling mislukt");
            }
        })
        .await?;

    Ok(())
}
