//! Priority queue and market sorting configuration.
//!
//! This module provides configurable settings for the priority-based execution system,
//! including expiration-based sorting, live game prioritization, liquidity constraints,
//! and profit threshold optimization.
//!
//! All settings can be overridden via environment variables while preserving sensible defaults.

use std::sync::OnceLock;

/// Priority system configuration loaded from environment
#[derive(Debug, Clone)]
pub struct PriorityConfig {
    /// Whether priority-based execution is enabled (default: false for backward compatibility)
    pub enabled: bool,

    /// Queue re-sort interval in seconds (default: 5)
    pub queue_sort_interval_secs: u64,

    /// Minimum liquidity per side in cents (default: 25000 = $250)
    pub min_liquidity_cents: u32,

    /// Maximum liquidity per side in cents (default: 250000 = $2500)
    pub max_liquidity_cents: u32,

    /// Minimum arbitrage percentage threshold (default: 1.0 = 1%)
    /// Only execute if profit_percent >= this value
    pub min_arb_percent: f64,

    /// Live game priority boost multiplier (default: 10.0)
    /// Higher values make live games sort higher in queue
    pub live_game_priority_boost: f64,

    /// Expiration weight for priority scoring (default: 1.0)
    /// Higher values prioritize markets expiring sooner
    pub expiration_weight: f64,

    /// Profit percentage weight for priority scoring (default: 2.0)
    /// Higher values prioritize higher profit opportunities
    pub profit_weight: f64,
}

impl Default for PriorityConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            queue_sort_interval_secs: 5,
            min_liquidity_cents: 25_000,  // $250
            max_liquidity_cents: 250_000, // $2,500
            min_arb_percent: 1.0,
            live_game_priority_boost: 10.0,
            expiration_weight: 1.0,
            profit_weight: 2.0,
        }
    }
}

impl PriorityConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            enabled: parse_env_bool("PRIORITY_MODE", false),
            queue_sort_interval_secs: parse_env_u64("QUEUE_SORT_INTERVAL_SECS", 5),
            min_liquidity_cents: parse_env_u32("MIN_LIQUIDITY_CENTS", 25_000),
            max_liquidity_cents: parse_env_u32("MAX_LIQUIDITY_CENTS", 250_000),
            min_arb_percent: parse_env_f64("MIN_ARB_PERCENT", 1.0),
            live_game_priority_boost: parse_env_f64("LIVE_PRIORITY_BOOST", 10.0),
            expiration_weight: parse_env_f64("EXPIRATION_WEIGHT", 1.0),
            profit_weight: parse_env_f64("PROFIT_WEIGHT", 2.0),
        }
    }

    /// Get the global priority config singleton
    #[allow(dead_code)]
    pub fn global() -> &'static PriorityConfig {
        static CONFIG: OnceLock<PriorityConfig> = OnceLock::new();
        CONFIG.get_or_init(PriorityConfig::from_env)
    }

    /// Check if a liquidity amount (in cents) meets the minimum constraint
    #[inline]
    pub fn meets_min_liquidity(&self, liquidity_cents: u32) -> bool {
        liquidity_cents >= self.min_liquidity_cents
    }

    /// Check if a liquidity amount (in cents) exceeds the maximum constraint
    #[inline]
    pub fn exceeds_max_liquidity(&self, liquidity_cents: u32) -> bool {
        liquidity_cents > self.max_liquidity_cents
    }

    /// Clamp liquidity to the configured range, returns (clamped_value, was_clamped)
    #[inline]
    #[allow(dead_code)]
    pub fn clamp_liquidity(&self, liquidity_cents: u32) -> (u32, bool) {
        if liquidity_cents < self.min_liquidity_cents {
            (0, true)  // Below minimum, reject
        } else if liquidity_cents > self.max_liquidity_cents {
            (self.max_liquidity_cents, true)  // Clamp to max
        } else {
            (liquidity_cents, false)
        }
    }

    /// Calculate the profit percentage from total cost in cents
    /// profit_percent = (100 - total_cost) / 100 * 100
    #[inline]
    pub fn profit_percent_from_cost(&self, total_cost_cents: u16) -> f64 {
        100.0 - total_cost_cents as f64
    }

    /// Check if profit percentage meets the minimum threshold
    #[inline]
    pub fn meets_min_profit(&self, profit_percent: f64) -> bool {
        profit_percent >= self.min_arb_percent
    }
}

// === Environment parsing helpers ===

fn parse_env_bool(key: &str, default: bool) -> bool {
    std::env::var(key)
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(default)
}

fn parse_env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn parse_env_u32(key: &str, default: u32) -> u32 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn parse_env_f64(key: &str, default: f64) -> f64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PriorityConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.queue_sort_interval_secs, 5);
        assert_eq!(config.min_liquidity_cents, 25_000);
        assert_eq!(config.max_liquidity_cents, 250_000);
        assert_eq!(config.min_arb_percent, 1.0);
    }

    #[test]
    fn test_liquidity_constraints() {
        let config = PriorityConfig::default();

        // Below minimum
        assert!(!config.meets_min_liquidity(20_000));
        let (clamped, was_clamped) = config.clamp_liquidity(20_000);
        assert_eq!(clamped, 0);
        assert!(was_clamped);

        // Within range
        assert!(config.meets_min_liquidity(50_000));
        let (clamped, was_clamped) = config.clamp_liquidity(50_000);
        assert_eq!(clamped, 50_000);
        assert!(!was_clamped);

        // Above maximum
        assert!(config.exceeds_max_liquidity(300_000));
        let (clamped, was_clamped) = config.clamp_liquidity(300_000);
        assert_eq!(clamped, 250_000);
        assert!(was_clamped);
    }

    #[test]
    fn test_profit_percent() {
        let config = PriorityConfig::default();

        // Total cost of 99 cents = 1% profit
        assert_eq!(config.profit_percent_from_cost(99), 1.0);
        assert!(config.meets_min_profit(1.0));

        // Total cost of 98 cents = 2% profit
        assert_eq!(config.profit_percent_from_cost(98), 2.0);
        assert!(config.meets_min_profit(2.0));

        // Total cost of 100 cents = 0% profit (no arb)
        assert_eq!(config.profit_percent_from_cost(100), 0.0);
        assert!(!config.meets_min_profit(0.0));
    }
}
