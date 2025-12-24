//! System configuration and league mapping definitions.
//!
//! This module contains all configuration constants, league mappings, and
//! environment variable parsing for the trading system.

/// Kalshi WebSocket URL
pub const KALSHI_WS_URL: &str = "wss://api.elections.kalshi.com/trade-api/ws/v2";

/// Kalshi REST API base URL
pub const KALSHI_API_BASE: &str = "https://api.elections.kalshi.com/trade-api/v2";

/// Polymarket WebSocket URL
pub const POLYMARKET_WS_URL: &str = "wss://ws-subscriptions-clob.polymarket.com/ws/market";

/// Gamma API base URL (Polymarket market data)
pub const GAMMA_API_BASE: &str = "https://gamma-api.polymarket.com";

/// Arb threshold: alert when total cost < this (e.g., 0.995 = 0.5% profit)
pub const ARB_THRESHOLD: f64 = 0.995;

/// Polymarket ping interval (seconds) - keep connection alive
/// Reduced from 30s to 15s for faster stale connection detection
pub const POLY_PING_INTERVAL_SECS: u64 = 15;

/// Kalshi API rate limit delay (milliseconds between requests)
/// Kalshi limit: 20 req/sec = 50ms minimum. We use 55ms to maximize speed.
pub const KALSHI_API_DELAY_MS: u64 = 55;

/// WebSocket reconnect delay (seconds) - reduced for faster recovery
pub const WS_RECONNECT_DELAY_SECS: u64 = 2;

/// Which leagues to monitor (empty slice = all)
pub const ENABLED_LEAGUES: &[&str] = &[];

/// Price logging enabled (set PRICE_LOGGING=1 to enable)
#[allow(dead_code)]
pub fn price_logging_enabled() -> bool {
    static CACHED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *CACHED.get_or_init(|| {
        std::env::var("PRICE_LOGGING")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false)
    })
}

/// League configuration for market discovery
#[derive(Debug, Clone)]
pub struct LeagueConfig {
    pub league_code: &'static str,
    pub poly_prefix: &'static str,
    pub kalshi_series_game: &'static str,
    pub kalshi_series_spread: Option<&'static str>,
    pub kalshi_series_total: Option<&'static str>,
    pub kalshi_series_btts: Option<&'static str>,
}

/// Get all supported leagues with their configurations
pub fn get_league_configs() -> Vec<LeagueConfig> {
    vec![
        // Major European leagues (full market types)
        LeagueConfig {
            league_code: "epl",
            poly_prefix: "epl",
            kalshi_series_game: "KXEPLGAME",
            kalshi_series_spread: Some("KXEPLSPREAD"),
            kalshi_series_total: Some("KXEPLTOTAL"),
            kalshi_series_btts: Some("KXEPLBTTS"),
        },
        LeagueConfig {
            league_code: "bundesliga",
            poly_prefix: "bun",
            kalshi_series_game: "KXBUNDESLIGAGAME",
            kalshi_series_spread: Some("KXBUNDESLIGASPREAD"),
            kalshi_series_total: Some("KXBUNDESLIGATOTAL"),
            kalshi_series_btts: Some("KXBUNDESLIGABTTS"),
        },
        LeagueConfig {
            league_code: "laliga",
            poly_prefix: "lal",
            kalshi_series_game: "KXLALIGAGAME",
            kalshi_series_spread: Some("KXLALIGASPREAD"),
            kalshi_series_total: Some("KXLALIGATOTAL"),
            kalshi_series_btts: Some("KXLALIGABTTS"),
        },
        LeagueConfig {
            league_code: "seriea",
            poly_prefix: "sea",
            kalshi_series_game: "KXSERIEAGAME",
            kalshi_series_spread: Some("KXSERIEASPREAD"),
            kalshi_series_total: Some("KXSERIEATOTAL"),
            kalshi_series_btts: Some("KXSERIEABTTS"),
        },
        LeagueConfig {
            league_code: "ligue1",
            poly_prefix: "fl1",
            kalshi_series_game: "KXLIGUE1GAME",
            kalshi_series_spread: Some("KXLIGUE1SPREAD"),
            kalshi_series_total: Some("KXLIGUE1TOTAL"),
            kalshi_series_btts: Some("KXLIGUE1BTTS"),
        },
        LeagueConfig {
            league_code: "ucl",
            poly_prefix: "ucl",
            kalshi_series_game: "KXUCLGAME",
            kalshi_series_spread: Some("KXUCLSPREAD"),
            kalshi_series_total: Some("KXUCLTOTAL"),
            kalshi_series_btts: Some("KXUCLBTTS"),
        },
        // Secondary European leagues (moneyline only)
        LeagueConfig {
            league_code: "uel",
            poly_prefix: "uel",
            kalshi_series_game: "KXUELGAME",
            kalshi_series_spread: None,
            kalshi_series_total: None,
            kalshi_series_btts: None,
        },
        LeagueConfig {
            league_code: "eflc",
            poly_prefix: "elc",
            kalshi_series_game: "KXEFLCHAMPIONSHIPGAME",
            kalshi_series_spread: None,
            kalshi_series_total: None,
            kalshi_series_btts: None,
        },
        // US Sports
        LeagueConfig {
            league_code: "nba",
            poly_prefix: "nba",
            kalshi_series_game: "KXNBAGAME",
            kalshi_series_spread: Some("KXNBASPREAD"),
            kalshi_series_total: Some("KXNBATOTAL"),
            kalshi_series_btts: None,
        },
        LeagueConfig {
            league_code: "wnba",
            poly_prefix: "wnba",
            kalshi_series_game: "KXWNBAGAME",
            kalshi_series_spread: None,  // Not confirmed on Kalshi
            kalshi_series_total: None,   // Not confirmed on Kalshi
            kalshi_series_btts: None,
        },
        LeagueConfig {
            league_code: "nfl",
            poly_prefix: "nfl",
            kalshi_series_game: "KXNFLGAME",
            kalshi_series_spread: Some("KXNFLSPREAD"),
            kalshi_series_total: Some("KXNFLTOTAL"),
            kalshi_series_btts: None,
        },
        LeagueConfig {
            league_code: "nhl",
            poly_prefix: "nhl",
            kalshi_series_game: "KXNHLGAME",
            kalshi_series_spread: Some("KXNHLSPREAD"),
            kalshi_series_total: Some("KXNHLTOTAL"),
            kalshi_series_btts: None,
        },
        LeagueConfig {
            league_code: "mlb",
            poly_prefix: "mlb",
            kalshi_series_game: "KXMLBGAME",
            kalshi_series_spread: Some("KXMLBSPREAD"),
            kalshi_series_total: Some("KXMLBTOTAL"),
            kalshi_series_btts: None,
        },
        LeagueConfig {
            league_code: "mls",
            poly_prefix: "mls",
            kalshi_series_game: "KXMLSGAME",
            kalshi_series_spread: None,
            kalshi_series_total: None,
            kalshi_series_btts: None,
        },
        LeagueConfig {
            league_code: "ncaaf",
            poly_prefix: "cfb",
            kalshi_series_game: "KXNCAAFGAME",
            kalshi_series_spread: Some("KXNCAAFSPREAD"),
            kalshi_series_total: Some("KXNCAAFTOTAL"),
            kalshi_series_btts: None,
        },
        // College Basketball
        LeagueConfig {
            league_code: "ncaamb",
            poly_prefix: "cbb",
            kalshi_series_game: "KXNCAAMBGAME",
            kalshi_series_spread: Some("KXNCAAMBSPREAD"),
            kalshi_series_total: Some("KXNCAAMBTOTAL"),
            kalshi_series_btts: None,
        },
        LeagueConfig {
            league_code: "ncaawb",
            poly_prefix: "wcbb",
            kalshi_series_game: "KXNCAAWBGAME",
            kalshi_series_spread: Some("KXNCAAWBSPREAD"),
            kalshi_series_total: Some("KXNCAAWBTOTAL"),
            kalshi_series_btts: None,
        },
    ]
}

/// Get config for a specific league
pub fn get_league_config(league: &str) -> Option<LeagueConfig> {
    get_league_configs()
        .into_iter()
        .find(|c| c.league_code == league || c.poly_prefix == league)
}

// =============================================================================
// CRYPTO MARKET CONFIGURATION
// =============================================================================

/// Crypto asset identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CryptoAsset {
    BTC,
    ETH,
    SOL,
    XRP,
    DOGE,
}

impl CryptoAsset {
    pub fn as_str(&self) -> &'static str {
        match self {
            CryptoAsset::BTC => "btc",
            CryptoAsset::ETH => "eth",
            CryptoAsset::SOL => "sol",
            CryptoAsset::XRP => "xrp",
            CryptoAsset::DOGE => "doge",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            CryptoAsset::BTC => "Bitcoin",
            CryptoAsset::ETH => "Ethereum",
            CryptoAsset::SOL => "Solana",
            CryptoAsset::XRP => "XRP",
            CryptoAsset::DOGE => "Dogecoin",
        }
    }
}

impl std::fmt::Display for CryptoAsset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str().to_uppercase())
    }
}

/// Crypto market timeframe
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CryptoTimeframe {
    /// 15-minute markets (Polymarket only)
    FifteenMin,
    /// Hourly markets (both platforms)
    Hourly,
    /// 4-hour markets
    FourHour,
    /// Daily markets (EOD settlement)
    Daily,
    /// Weekly markets
    Weekly,
}

impl CryptoTimeframe {
    pub fn as_str(&self) -> &'static str {
        match self {
            CryptoTimeframe::FifteenMin => "15m",
            CryptoTimeframe::Hourly => "1h",
            CryptoTimeframe::FourHour => "4h",
            CryptoTimeframe::Daily => "1d",
            CryptoTimeframe::Weekly => "1w",
        }
    }

    /// Scan interval in milliseconds for this timeframe
    pub fn scan_interval_ms(&self) -> u64 {
        match self {
            CryptoTimeframe::FifteenMin => 500,   // Very fast - 0.5 second
            CryptoTimeframe::Hourly => 1000,      // Fast - 1 second
            CryptoTimeframe::FourHour => 5000,    // Normal - 5 seconds
            CryptoTimeframe::Daily => 10000,      // Slow - 10 seconds
            CryptoTimeframe::Weekly => 30000,     // Very slow - 30 seconds
        }
    }
}

/// Platform availability for crypto markets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptoPlatform {
    /// Only available on Kalshi (same-platform arb between brackets)
    KalshiOnly,
    /// Only available on Polymarket (same-platform YES/NO arb)
    PolymarketOnly,
    /// Available on both platforms (cross-platform arb)
    Both,
}

/// Configuration for a crypto market series
#[derive(Debug, Clone)]
pub struct CryptoMarketConfig {
    /// Asset being traded
    pub asset: CryptoAsset,
    /// Market timeframe
    pub timeframe: CryptoTimeframe,
    /// Which platforms have this market
    pub platform: CryptoPlatform,
    /// Kalshi series ticker (if applicable)
    pub kalshi_series: Option<&'static str>,
    /// Polymarket slug pattern (if applicable)
    /// Use {asset}, {date}, {timestamp} as placeholders
    pub poly_slug_pattern: Option<&'static str>,
    /// Whether this market has multiple price brackets (Kalshi hourly)
    pub has_brackets: bool,
}

/// Get all supported crypto market configurations
pub fn get_crypto_configs() -> Vec<CryptoMarketConfig> {
    vec![
        // === KALSHI HOURLY PRICE BRACKETS (Same-platform arb) ===
        // BTC and ETH have hourly markets with multiple price brackets
        CryptoMarketConfig {
            asset: CryptoAsset::BTC,
            timeframe: CryptoTimeframe::Hourly,
            platform: CryptoPlatform::KalshiOnly,
            kalshi_series: Some("KXBTCD"),  // Bitcoin daily/hourly
            poly_slug_pattern: None,
            has_brackets: true,
        },
        CryptoMarketConfig {
            asset: CryptoAsset::ETH,
            timeframe: CryptoTimeframe::Hourly,
            platform: CryptoPlatform::KalshiOnly,
            kalshi_series: Some("KXETHD"),  // Ethereum daily/hourly
            poly_slug_pattern: None,
            has_brackets: true,
        },

        // === POLYMARKET 15-MINUTE MARKETS (Same-platform arb) ===
        // Up/Down markets that settle every 15 minutes
        CryptoMarketConfig {
            asset: CryptoAsset::BTC,
            timeframe: CryptoTimeframe::FifteenMin,
            platform: CryptoPlatform::PolymarketOnly,
            kalshi_series: None,
            poly_slug_pattern: Some("btc-updown-15m-{timestamp}"),
            has_brackets: false,
        },
        CryptoMarketConfig {
            asset: CryptoAsset::ETH,
            timeframe: CryptoTimeframe::FifteenMin,
            platform: CryptoPlatform::PolymarketOnly,
            kalshi_series: None,
            poly_slug_pattern: Some("eth-updown-15m-{timestamp}"),
            has_brackets: false,
        },
        CryptoMarketConfig {
            asset: CryptoAsset::SOL,
            timeframe: CryptoTimeframe::FifteenMin,
            platform: CryptoPlatform::PolymarketOnly,
            kalshi_series: None,
            poly_slug_pattern: Some("sol-updown-15m-{timestamp}"),
            has_brackets: false,
        },
        CryptoMarketConfig {
            asset: CryptoAsset::XRP,
            timeframe: CryptoTimeframe::FifteenMin,
            platform: CryptoPlatform::PolymarketOnly,
            kalshi_series: None,
            poly_slug_pattern: Some("xrp-updown-15m-{timestamp}"),
            has_brackets: false,
        },

        // === POLYMARKET HOURLY MARKETS ===
        CryptoMarketConfig {
            asset: CryptoAsset::BTC,
            timeframe: CryptoTimeframe::Hourly,
            platform: CryptoPlatform::PolymarketOnly,
            kalshi_series: None,
            poly_slug_pattern: Some("btc-hourly-{date}"),
            has_brackets: false,
        },
        CryptoMarketConfig {
            asset: CryptoAsset::ETH,
            timeframe: CryptoTimeframe::Hourly,
            platform: CryptoPlatform::PolymarketOnly,
            kalshi_series: None,
            poly_slug_pattern: Some("eth-hourly-{date}"),
            has_brackets: false,
        },

        // === CROSS-PLATFORM DAILY MARKETS ===
        // These exist on both platforms with matching settlement times
        CryptoMarketConfig {
            asset: CryptoAsset::BTC,
            timeframe: CryptoTimeframe::Daily,
            platform: CryptoPlatform::Both,
            kalshi_series: Some("KXBTCD"),
            poly_slug_pattern: Some("bitcoin-above-{price}-{date}"),
            has_brackets: false,
        },
    ]
}

/// Get crypto configs filtered by asset
pub fn get_crypto_configs_for_asset(asset: CryptoAsset) -> Vec<CryptoMarketConfig> {
    get_crypto_configs()
        .into_iter()
        .filter(|c| c.asset == asset)
        .collect()
}

/// Get crypto configs filtered by platform
pub fn get_crypto_configs_for_platform(platform: CryptoPlatform) -> Vec<CryptoMarketConfig> {
    get_crypto_configs()
        .into_iter()
        .filter(|c| c.platform == platform)
        .collect()
}

/// Get the fastest scan interval needed for enabled crypto markets
pub fn get_crypto_fast_scan_interval_ms() -> u64 {
    get_crypto_configs()
        .iter()
        .map(|c| c.timeframe.scan_interval_ms())
        .min()
        .unwrap_or(1000)
}

/// Check if crypto markets are enabled via environment variable
pub fn crypto_enabled() -> bool {
    static CACHED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *CACHED.get_or_init(|| {
        std::env::var("CRYPTO_ENABLED")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(true)  // Enabled by default - set CRYPTO_ENABLED=0 to disable
    })
}

// =============================================================================
// STOCK INDEX MARKET CONFIGURATION (Kalshi only - same platform arb)
// =============================================================================

/// Stock index identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StockIndex {
    SP500,
    Nasdaq100,
}

impl StockIndex {
    pub fn as_str(&self) -> &'static str {
        match self {
            StockIndex::SP500 => "spx",
            StockIndex::Nasdaq100 => "ndx",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            StockIndex::SP500 => "S&P 500",
            StockIndex::Nasdaq100 => "NASDAQ-100",
        }
    }

    pub fn kalshi_series(&self) -> &'static str {
        match self {
            StockIndex::SP500 => "INXD",
            StockIndex::Nasdaq100 => "NASDAQ100D",
        }
    }
}

/// Get stock index market configurations
pub fn get_index_configs() -> Vec<(StockIndex, &'static str)> {
    vec![
        (StockIndex::SP500, "INXD"),
        (StockIndex::Nasdaq100, "NASDAQ100D"),
    ]
}