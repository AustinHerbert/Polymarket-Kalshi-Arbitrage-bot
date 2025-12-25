//! Spread bot configuration with verified Polymarket token IDs
//!
//! Token IDs verified from Polymarket Gamma API - December 24, 2025

use anyhow::Result;

/// Spread farming configuration
#[derive(Debug, Clone)]
pub struct SpreadFarmingConfig {
    pub dry_run: bool,
    pub spread_bps: u32,
    pub max_position_per_market_usd: f64,
    pub update_interval_ms: u64,
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
                .unwrap_or(200),
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

/// Market configuration with REAL verified token IDs
#[derive(Debug, Clone)]
pub struct MarketConfig {
    pub name: &'static str,
    pub description: &'static str,
    pub yes_token: &'static str,
    pub no_token: &'static str,
}

/// Target markets - verified December 24, 2025 from Polymarket Gamma API
pub const TARGET_MARKETS: &[MarketConfig] = &[
    // Bitcoin $95K - 10.5% probability, best for two-sided MM
    MarketConfig {
        name: "btc_95k",
        description: "BTC reach $95K by Dec 31, 2025",
        yes_token: "96867039153990962337615945364940037915308440159720050466327100056373743698980",
        no_token: "41111130186959012758904331542255682579981367443843763371097525816880891318195",
    },
    // Bitcoin $75K dip - 3.45% probability
    MarketConfig {
        name: "btc_75k_dip",
        description: "BTC dip to $75K by Dec 31, 2025",
        yes_token: "4381437605923304671404496289379268024880920729170055186356445875810529628422",
        no_token: "45885466573239033848345089284645725729633512376610890353339136835959570108782",
    },
    // Ethereum $5K - 0.25% probability
    MarketConfig {
        name: "eth_5k",
        description: "ETH hit $5K by Dec 31",
        yes_token: "96638575418189284731461006608299472691495172793493062289677934142140726427384",
        no_token: "101047384638948889500274126169139867032724036159322634390400440566762028472215",
    },
];
