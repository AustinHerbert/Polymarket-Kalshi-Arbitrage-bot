//! Configuration for spread farming bot

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Main configuration for spread farming strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadFarmingConfig {
    /// Enable/disable dry run mode (default: true)
    pub dry_run: bool,
    
    /// Spread in basis points (100 bps = 1%)
    /// Default: 400 bps = 4% spread (bid at -2%, ask at +2%)
    pub spread_bps: u32,
    
    /// Maximum position size per market in USD
    /// Default: $500 per market
    pub max_position_per_market_usd: f64,
    
    /// How often to update orders (milliseconds)
    /// Default: 5000ms = 5 seconds
    pub update_interval_ms: u64,
    
    /// Minimum order size in USD (Polymarket minimum is ~$1)
    pub min_order_size_usd: f64,
}

impl SpreadFarmingConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            dry_run: std::env::var("SPREAD_DRY_RUN")
                .map(|v| v == "1" || v == "true")
                .unwrap_or(true),
            
            spread_bps: std::env::var("SPREAD_BPS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(400),  // 4% default spread
            
            max_position_per_market_usd: std::env::var("MAX_POSITION_PER_MARKET")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(500.0),
            
            update_interval_ms: std::env::var("UPDATE_INTERVAL_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5000),
            
            min_order_size_usd: std::env::var("MIN_ORDER_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(2.0),
        })
    }
}

/// Target markets for spread farming
///
/// Selection criteria:
/// 1. HIGH LIQUIDITY - Need tight existing spreads and good depth
/// 2. CRYPTO MARKETS ONLY - More predictable, less manipulation than sports
/// 3. REASONABLE VOLATILITY - Not too stable (no spread), not too volatile (inventory risk)
/// 4. CLEAR OUTCOMES - Binary yes/no markets with definitive resolution
///
/// Markets chosen (verified December 2025):
/// Source: https://polymarket.com/predictions/crypto-prices
pub const TARGET_MARKETS: &[(&str, &str)] = &[
    // Bitcoin price markets (VERIFIED - $147M volume)
    ("what-price-will-bitcoin-hit-in-2025", "Bitcoin 2025 Price Prediction"),

    // Ethereum price markets (VERIFIED - $64M volume)
    ("what-price-will-ethereum-hit-in-2025", "Ethereum 2025 Price Prediction"),
];

/// Market data structure for fetching token IDs
#[derive(Debug, Clone)]
pub struct MarketData {
    pub slug: String,
    pub description: String,
    pub token_ids: Vec<String>,  // Multiple outcomes per market
}
