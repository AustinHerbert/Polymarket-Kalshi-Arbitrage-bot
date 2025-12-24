//! Polymarket Spread Farming Bot
//!
//! A market making bot that provides liquidity to Polymarket crypto prediction markets
//! by placing limit orders on both sides of the orderbook and capturing the spread.

mod config;
mod market_maker;
mod order_manager;
mod types;

use anyhow::{Context, Result};
use std::sync::Arc;
use tracing::{info, warn};

use config::{SpreadFarmingConfig, TARGET_MARKETS};
use market_maker::MarketMaker;
use shared_market_client::{PolymarketAsyncClient, PreparedCreds, SharedAsyncClient};

/// Polygon chain ID
const POLYGON_CHAIN_ID: u64 = 137;
/// Polymarket CLOB API host
const POLY_CLOB_HOST: &str = "https://clob.polymarket.com";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("spread_farming_bot=info".parse().unwrap()),
        )
        .init();

    info!("🌾 Polymarket Spread Farming Bot v0.1.0");
    info!("   Strategy: Market Making (Liquidity Provision)");
    info!("   Target: Polymarket Crypto Markets");

    // Load configuration
    let config = SpreadFarmingConfig::from_env()?;
    info!("   Mode: {}", if config.dry_run { "DRY RUN" } else { "LIVE EXECUTION" });
    info!("   Spread: {} bps", config.spread_bps);
    info!("   Max position per market: ${}", config.max_position_per_market_usd);
    info!("   Update interval: {}ms", config.update_interval_ms);

    if !config.dry_run {
        warn!("⚠️  LIVE EXECUTION MODE - Real money at risk!");
    }

    // Load Polymarket credentials
    dotenvy::dotenv().ok();
    let poly_private_key = std::env::var("POLY_PRIVATE_KEY")
        .context("POLY_PRIVATE_KEY not set")?;
    let poly_funder = std::env::var("POLY_FUNDER")
        .context("POLY_FUNDER not set (your wallet address)")?;

    // Create Polymarket client
    info!("[POLYMARKET] Creating client and deriving API credentials...");
    let poly_async_client = PolymarketAsyncClient::new(
        POLY_CLOB_HOST,
        POLYGON_CHAIN_ID,
        &poly_private_key,
        &poly_funder,
    )?;

    let api_creds = poly_async_client.derive_api_key(0).await?;
    let prepared_creds = PreparedCreds::from_api_creds(&api_creds)?;
    let poly_client = Arc::new(SharedAsyncClient::new(
        poly_async_client,
        prepared_creds,
        POLYGON_CHAIN_ID,
    ));

    info!("[POLYMARKET] Client ready for {}", &poly_funder[..10]);

    // Display target markets with REAL token IDs
    info!("📊 Target markets:");
    for market in TARGET_MARKETS {
        info!("   ✓ {} [{}]", market.description, market.name);
        info!("     YES: {}...", &market.yes_token[..30]);
        info!("     NO:  {}...", &market.no_token[..30]);
    }

    // Create market maker engine
    let market_maker = Arc::new(MarketMaker::new(poly_client.clone(), config.clone()));

    // Initialize market maker (verify orderbooks exist)
    market_maker.initialize().await?;

    info!("✅ Spread farming bot initialized");
    info!("   Starting market making engine...\n");

    // Run market making loop
    market_maker.run().await?;

    Ok(())
}
