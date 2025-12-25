//! Trading Dashboard
//!
//! Unified web dashboard for monitoring all trading bots.
//! Serves on port 8080 by default.

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("dashboard=info".parse().unwrap()),
        )
        .init();

    let port = std::env::var("DASHBOARD_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080u16);

    info!("📊 Trading Dashboard v2.0.0");
    info!("   Port: {}", port);
    info!("   Status: PLACEHOLDER");

    // TODO: Implement web server with axum
    // - GET /           -> Dashboard home
    // - GET /arb-bot    -> Arb bot status
    // - GET /spread-bot -> Spread bot status
    // - GET /api/status -> JSON status endpoint

    info!("Dashboard placeholder - not yet implemented");
    info!("Run arb-bot or spread-bot directly for now");

    Ok(())
}
