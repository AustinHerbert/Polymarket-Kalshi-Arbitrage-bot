//! Configuration for spread farming bot
//!
//! Token IDs verified from Polymarket Gamma API - December 24, 2025

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Main configuration for spread farming strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadFarmingConfig {
    /// Enable/disable dry run mode (default: true)
    pub dry_run: bool,

    /// Spread in basis points (100 bps = 1%)
    /// Default: 200 bps = 2% spread (bid at -1%, ask at +1%)
    pub spread_bps: u32,

    /// Maximum position size per market in USD
    pub max_position_per_market_usd: f64,

    /// How often to update orders (milliseconds)
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
                .unwrap_or(200),  // 2% default spread

            max_position_per_market_usd: std::env::var("MAX_POSITION_PER_MARKET")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(50.0),

            update_interval_ms: std::env::var("UPDATE_INTERVAL_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10000),

            min_order_size_usd: std::env::var("MIN_ORDER_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(2.0),
        })
    }
}

/// Market configuration with REAL token IDs from Polymarket
#[derive(Debug, Clone)]
pub struct MarketConfig {
    /// Short name for the market
    pub name: &'static str,
    /// Human-readable description
    pub description: &'static str,
    /// YES outcome token ID (from clobTokenIds[0])
    pub yes_token: &'static str,
    /// NO outcome token ID (from clobTokenIds[1])
    pub no_token: &'static str,
}

/// Target markets with VERIFIED token IDs
///
/// These are the actual sub-markets with orderbooks.
/// Token IDs fetched from: https://gamma-api.polymarket.com/events
/// Verified: December 24, 2025
pub const TARGET_MARKETS: &[MarketConfig] = &[
    // ============================================================
    // BITCOIN MARKETS (Parent event: $147M volume)
    // ============================================================

    // BEST: 10.5% probability - good for two-sided market making
    MarketConfig {
        name: "btc_95k",
        description: "Will Bitcoin reach $95,000 by Dec 31, 2025?",
        yes_token: "96867039153990962337615945364940037915308440159720050466327100056373743698980",
        no_token: "41111130186959012758904331542255682579981367443843763371097525816880891318195",
    },

    // 3.45% probability - lower but still tradeable
    MarketConfig {
        name: "btc_75k_dip",
        description: "Will Bitcoin dip to $75,000 by Dec 31, 2025?",
        yes_token: "4381437605923304671404496289379268024880920729170055186356445875810529628422",
        no_token: "45885466573239033848345089284645725729633512376610890353339136835959570108782",
    },

    // ============================================================
    // ETHEREUM MARKETS (Parent event: $64M volume)
    // ============================================================

    // 0.25% probability
    MarketConfig {
        name: "eth_5k",
        description: "Will Ethereum hit $5,000 by Dec 31?",
        yes_token: "96638575418189284731461006608299472691495172793493062289677934142140726427384",
        no_token: "101047384638948889500274126169139867032724036159322634390400440566762028472215",
    },
];

/// Get all target markets
pub fn get_target_markets() -> &'static [MarketConfig] {
    TARGET_MARKETS
}

/// Get market by name
pub fn get_market(name: &str) -> Option<&'static MarketConfig> {
    TARGET_MARKETS.iter().find(|m| m.name == name)
}
