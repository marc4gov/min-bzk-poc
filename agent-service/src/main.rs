use agent_service::create_app;
use anyhow::Result;
use std::net::SocketAddr;
use std::path::Path;

/// Laadt `agent-service/.env` (compile-time pad), daarna optioneel `.env` in cwd.
fn load_dotenv() {
    let crate_env = Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
    if crate_env.exists() {
        if let Err(e) = dotenvy::from_path(&crate_env) {
            eprintln!(
                "waarschuwing: .env niet geladen ({}): {}",
                crate_env.display(),
                e
            );
        }
    }
    let _ = dotenvy::dotenv();
}

#[tokio::main]
async fn main() -> Result<()> {
    load_dotenv();

    if let Ok(url) = std::env::var("PII_PRIVACY_FILTER_URL") {
        if !url.trim().is_empty() {
            eprintln!("PII_PRIVACY_FILTER_URL={}", url.trim());
        }
    }
    if std::env::var("MESH_RESEARCH_HTTP")
        .map(|v| matches!(v.as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
    {
        eprintln!("MESH_RESEARCH_HTTP=1 — ResearchExpert gebruikt GET-previews");
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let app = create_app().await?;

    let port = std::env::var("MESH_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("Agent service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
