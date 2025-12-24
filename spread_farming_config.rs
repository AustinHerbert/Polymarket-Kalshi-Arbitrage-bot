//! Configuration for spread farming bot
//!
//! Updated: December 24, 2025
//! Token IDs verified from Polymarket Gamma API

use anyhow::Result;
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

/// Individual market configuration with verified token IDs
#[derive(Debug, Clone)]
pub struct MarketConfig {
    pub name: &'static str,
    pub slug: &'static str,
    pub description: &'static str,
    pub condition_id: &'static str,
    pub yes_token: &'static str,
    pub no_token: &'static str,
    /// Current mid-price (updated Dec 24, 2025)
    pub initial_price: f64,
}

/// Active crypto markets for spread farming
///
/// Selection criteria:
/// 1. Active = true, Closed = false, OrderBook = true
/// 2. Mid-range probability preferred (5-50%) for better two-sided liquidity
/// 3. High volume parent events for more trading activity
///
/// Source: Polymarket Gamma API (verified December 24, 2025)
pub const TARGET_MARKETS: &[MarketConfig] = &[
    // ============================================================
    // BITCOIN MARKETS (Parent: $147M volume)
    // ============================================================

    // BEST MARKET: Mid-range probability, good for two-sided MM
    MarketConfig {
        name: "btc_95k",
        slug: "will-bitcoin-reach-95000-by-december-31-2025-818-596-821-318-841",
        description: "Will Bitcoin reach $95,000 by December 31, 2025?",
        condition_id: "0xe1efac87d70a9556b222624def73e3d7adc7b4513c781867c6c7aa105ff1d9b5",
        yes_token: "96867039153990962337615945364940037915308440159720050466327100056373743698980",
        no_token: "41111130186959012758904331542255682579981367443843763371097525816880891318195",
        initial_price: 0.105,  // 10.5% YES
    },

    // Secondary: Lower probability dip market
    MarketConfig {
        name: "btc_75k_dip",
        slug: "will-bitcoin-dip-to-75000-by-december-31-2025-515",
        description: "Will Bitcoin dip to $75,000 by December 31, 2025?",
        condition_id: "0x4f20debe0c978a8cfffa20cc3019dd3740c945bb577f12c0d4b8c82c4bf0647f",
        yes_token: "4381437605923304671404496289379268024880920729170055186356445875810529628422",
        no_token: "45885466573239033848345089284645725729633512376610890353339136835959570108782",
        initial_price: 0.0345,  // 3.45% YES
    },

    // Low probability upside
    MarketConfig {
        name: "btc_115k",
        slug: "will-bitcoin-reach-115000-by-december-31-2025-746",
        description: "Will Bitcoin reach $115,000 by December 31, 2025?",
        condition_id: "0xc8f19832fd11ad8968e947c4e4fbed6059075efb73f9d1c9133ce6235a291fd0",
        yes_token: "43065048993201852298348000614811881300559249848584277889292802783823995424800",
        no_token: "114876740724049589622012682611965033273380494148463897971149069226393068552157",
        initial_price: 0.004,  // 0.4% YES
    },

    // Low probability dip
    MarketConfig {
        name: "btc_65k_dip",
        slug: "will-bitcoin-dip-to-65000-by-december-31-2025-724",
        description: "Will Bitcoin dip to $65,000 by December 31, 2025?",
        condition_id: "0x03a914dc0a98b109dfbcb6e8f8acfdf46c9a06fc6eac38f842b354983541a9e2",
        yes_token: "9544795897157931749789672600393215554463936120262788041377651439138297023248",
        no_token: "10486838757381743216680246160801049866550822572575063891923373335185432858811",
        initial_price: 0.0065,  // 0.65% YES
    },

    // ============================================================
    // ETHEREUM MARKETS (Parent: $64M volume)
    // ============================================================

    MarketConfig {
        name: "eth_5k",
        slug: "will-ethereum-hit-5000-by-december-31-796-376",
        description: "Will Ethereum hit $5,000 by December 31?",
        condition_id: "0xeeb92ddd77d671d1f9ea1f5727208d1fa91896a6d1f657fdcd7c9d578fe5c46c",
        yes_token: "96638575418189284731461006608299472691495172793493062289677934142140726427384",
        no_token: "101047384638948889500274126169139867032724036159322634390400440566762028472215",
        initial_price: 0.0025,  // 0.25% YES
    },

    MarketConfig {
        name: "eth_6k",
        slug: "will-ethereum-hit-6000-by-december-31-843-315",
        description: "Will Ethereum hit $6,000 by December 31?",
        condition_id: "0x9c08beafa73308625ad24e270e44d9edd634585f2424eea8df262a05a2337613",
        yes_token: "81425397446351426621727450375895254607015946433493851514616485979278175609514",
        no_token: "796584219710508341356184389057648832166516548136309631230564122389735606484",
        initial_price: 0.0015,  // 0.15% YES
    },

    MarketConfig {
        name: "eth_7k",
        slug: "will-ethereum-hit-7000-by-december-31-733-852",
        description: "Will Ethereum hit $7,000 by December 31?",
        condition_id: "0xb7f9d3f61910ecf7a5e6e46f487a039e91a92b68585f05ec9b22cb34d3804bbd",
        yes_token: "66776654258002694324853229694339306737497274534009979978324520345248856607706",
        no_token: "34130531637570468522541549058959039696296775081849932071066342884622778848556",
        initial_price: 0.0015,  // 0.15% YES
    },

    MarketConfig {
        name: "eth_1k_dip",
        slug: "will-ethereum-dip-to-1000-by-december-31-498-391-185-526-541-347-589",
        description: "Will Ethereum dip to $1,000 by December 31?",
        condition_id: "0x15e731302e61491cc8aafbcfb77febb1fdd58ea1525737a01f5e763d29e4588e",
        yes_token: "82510958209410597815455827415233906018439939153028999177265735407231980907130",
        no_token: "94060151012037153553135800277530835829038186375440423105871589646596848601697",
        initial_price: 0.0025,  // 0.25% YES
    },
];

/// Get all active target markets
pub fn get_target_markets() -> &'static [MarketConfig] {
    TARGET_MARKETS
}

/// Get a specific market by name
pub fn get_market(name: &str) -> Option<&'static MarketConfig> {
    TARGET_MARKETS.iter().find(|m| m.name == name)
}

/// Polymarket API endpoints
pub const GAMMA_API_BASE: &str = "https://gamma-api.polymarket.com";
pub const CLOB_API_BASE: &str = "https://clob.polymarket.com";
pub const POLYMARKET_WS_URL: &str = "wss://ws-subscriptions-clob.polymarket.com/ws/market";

/// Polygon chain ID
pub const POLYGON_CHAIN_ID: u64 = 137;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_config() {
        let markets = get_target_markets();
        assert!(!markets.is_empty());

        for market in markets {
            assert!(!market.yes_token.is_empty());
            assert!(!market.no_token.is_empty());
            assert!(market.yes_token != market.no_token);
        }
    }

    #[test]
    fn test_get_market() {
        let btc = get_market("btc_95k");
        assert!(btc.is_some());
        assert_eq!(btc.unwrap().initial_price, 0.105);
    }
}
