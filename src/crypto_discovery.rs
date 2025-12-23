//! Crypto market discovery and same-platform arbitrage detection.
//!
//! This module handles discovery of cryptocurrency price markets on Kalshi and Polymarket,
//! with support for:
//! - Kalshi hourly price brackets (same-platform arb between brackets)
//! - Polymarket 15-minute up/down markets (same-platform YES/NO arb)
//! - Cross-platform daily BTC markets

use anyhow::Result;
use governor::{Quota, RateLimiter, state::NotKeyed, clock::DefaultClock, middleware::NoOpMiddleware};
use serde::{Serialize, Deserialize};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Semaphore;
use tracing::{info, warn, debug};

use crate::config::{
    CryptoAsset, CryptoPlatform,
    get_crypto_configs, crypto_enabled, GAMMA_API_BASE,
};
use crate::kalshi::KalshiApiClient;
use crate::types::{MarketType, MarketCategory};

/// Kalshi rate limit for crypto discovery (conservative)
const KALSHI_RATE_LIMIT_PER_SEC: u32 = 2;

/// Max concurrent Kalshi API requests
const KALSHI_CONCURRENCY: usize = 1;

/// Polymarket Gamma API rate limit
const GAMMA_RATE_LIMIT_PER_SEC: u32 = 10;

/// Max concurrent Gamma API requests
const GAMMA_CONCURRENCY: usize = 5;

/// Cache TTL for crypto markets (15 minutes - crypto is fast-moving)
const CRYPTO_CACHE_TTL_SECS: u64 = 15 * 60;

/// Crypto discovery cache file
const CRYPTO_CACHE_PATH: &str = ".crypto_discovery_cache.json";

/// Type alias for rate limiter
type RateLimit = RateLimiter<NotKeyed, governor::state::InMemoryState, DefaultClock, NoOpMiddleware>;

/// A discovered crypto market for same-platform or cross-platform arbitrage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoMarket {
    /// Unique identifier
    pub market_id: Arc<str>,
    /// Asset being traded (BTC, ETH, etc.)
    pub asset: String,
    /// Market timeframe
    pub timeframe: String,
    /// Platform this market is on
    pub platform: String,
    /// Market description
    pub description: Arc<str>,
    /// Kalshi market ticker (if applicable)
    pub kalshi_ticker: Option<Arc<str>>,
    /// Polymarket slug (if applicable)
    pub poly_slug: Option<Arc<str>>,
    /// Polymarket YES token (if applicable)
    pub poly_yes_token: Option<Arc<str>>,
    /// Polymarket NO token (if applicable)
    pub poly_no_token: Option<Arc<str>>,
    /// Price threshold for this bracket (e.g., "above $95,000")
    pub price_threshold: Option<f64>,
    /// Settlement time as Unix timestamp
    pub settlement_time_secs: Option<u64>,
    /// Market category
    pub category: MarketCategory,
    /// Market type
    pub market_type: MarketType,
}

/// A Kalshi bracket group for same-platform arbitrage
/// Multiple adjacent brackets that can be arbitraged together
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KalshiBracketGroup {
    /// Asset (BTC, ETH, etc.)
    pub asset: String,
    /// Event ticker (e.g., "KXBTCD-25DEC23-12PM")
    pub event_ticker: Arc<str>,
    /// Human-readable description
    pub description: Arc<str>,
    /// Settlement time as Unix timestamp
    pub settlement_time_secs: Option<u64>,
    /// List of bracket markets (sorted by price threshold)
    pub brackets: Vec<KalshiBracket>,
}

/// A single bracket within a Kalshi bracket group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KalshiBracket {
    /// Market ticker
    pub ticker: Arc<str>,
    /// Price threshold (e.g., 95000.0 for "above $95,000")
    pub threshold: f64,
    /// Current YES ask price in cents
    pub yes_ask_cents: Option<u16>,
    /// Current NO ask price in cents
    pub no_ask_cents: Option<u16>,
}

/// A Polymarket up/down market for same-platform arb
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolyUpDownMarket {
    /// Asset (BTC, ETH, etc.)
    pub asset: String,
    /// Market slug
    pub slug: Arc<str>,
    /// YES token address
    pub yes_token: Arc<str>,
    /// NO token address
    pub no_token: Arc<str>,
    /// Settlement time as Unix timestamp
    pub settlement_time_secs: Option<u64>,
    /// Description
    pub description: Arc<str>,
}

/// Result of crypto market discovery
#[derive(Debug, Default)]
pub struct CryptoDiscoveryResult {
    /// Kalshi bracket groups (for same-platform arb)
    pub kalshi_bracket_groups: Vec<KalshiBracketGroup>,
    /// Polymarket up/down markets (for same-platform arb)
    pub poly_updown_markets: Vec<PolyUpDownMarket>,
    /// Cross-platform matched pairs
    pub cross_platform_pairs: Vec<CryptoMarket>,
    /// Total markets found
    pub total_markets: usize,
    /// Discovery errors
    pub errors: Vec<String>,
}

/// Cache for crypto discovery results
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CryptoCache {
    timestamp_secs: u64,
    kalshi_bracket_groups: Vec<KalshiBracketGroup>,
    poly_updown_markets: Vec<PolyUpDownMarket>,
}

impl CryptoCache {
    fn is_expired(&self) -> bool {
        let now = current_unix_secs();
        now.saturating_sub(self.timestamp_secs) > CRYPTO_CACHE_TTL_SECS
    }

    fn age_secs(&self) -> u64 {
        current_unix_secs().saturating_sub(self.timestamp_secs)
    }
}

fn current_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Client for discovering crypto markets
pub struct CryptoDiscoveryClient {
    kalshi: Arc<KalshiApiClient>,
    http: reqwest::Client,
    kalshi_limiter: Arc<RateLimit>,
    kalshi_semaphore: Arc<Semaphore>,
    gamma_limiter: Arc<RateLimit>,
    gamma_semaphore: Arc<Semaphore>,
}

impl CryptoDiscoveryClient {
    pub fn new(kalshi: KalshiApiClient) -> Self {
        let kalshi_quota = Quota::per_second(NonZeroU32::new(KALSHI_RATE_LIMIT_PER_SEC).unwrap());
        let gamma_quota = Quota::per_second(NonZeroU32::new(GAMMA_RATE_LIMIT_PER_SEC).unwrap());

        Self {
            kalshi: Arc::new(kalshi),
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("Failed to build HTTP client"),
            kalshi_limiter: Arc::new(RateLimiter::direct(kalshi_quota)),
            kalshi_semaphore: Arc::new(Semaphore::new(KALSHI_CONCURRENCY)),
            gamma_limiter: Arc::new(RateLimiter::direct(gamma_quota)),
            gamma_semaphore: Arc::new(Semaphore::new(GAMMA_CONCURRENCY)),
        }
    }

    /// Load cache from disk
    async fn load_cache() -> Option<CryptoCache> {
        let data = tokio::fs::read_to_string(CRYPTO_CACHE_PATH).await.ok()?;
        serde_json::from_str(&data).ok()
    }

    /// Save cache to disk
    async fn save_cache(cache: &CryptoCache) -> Result<()> {
        let data = serde_json::to_string_pretty(cache)?;
        tokio::fs::write(CRYPTO_CACHE_PATH, data).await?;
        Ok(())
    }

    /// Discover all crypto markets
    pub async fn discover_all(&self) -> CryptoDiscoveryResult {
        if !crypto_enabled() {
            info!("₿ Crypto discovery disabled (set CRYPTO_ENABLED=1 to enable)");
            return CryptoDiscoveryResult::default();
        }

        info!("₿ Starting crypto market discovery...");

        // Try to load cache
        if let Some(cache) = Self::load_cache().await {
            if !cache.is_expired() {
                info!("₿ Loaded {} bracket groups, {} updown markets from cache (age: {}s)",
                      cache.kalshi_bracket_groups.len(),
                      cache.poly_updown_markets.len(),
                      cache.age_secs());
                return CryptoDiscoveryResult {
                    kalshi_bracket_groups: cache.kalshi_bracket_groups,
                    poly_updown_markets: cache.poly_updown_markets,
                    cross_platform_pairs: vec![],
                    total_markets: 0,
                    errors: vec![],
                };
            }
        }

        let mut result = CryptoDiscoveryResult::default();

        // Get crypto configs
        let configs = get_crypto_configs();

        // Discover Kalshi bracket markets
        for config in configs.iter().filter(|c| c.platform != CryptoPlatform::PolymarketOnly) {
            if let Some(series) = config.kalshi_series {
                match self.discover_kalshi_brackets(config.asset, series).await {
                    Ok(groups) => {
                        info!("  ₿ {} {}: {} bracket groups found",
                              config.asset, config.timeframe.as_str(), groups.len());
                        result.kalshi_bracket_groups.extend(groups);
                    }
                    Err(e) => {
                        result.errors.push(format!("Kalshi {} {}: {}",
                            config.asset.as_str(), config.timeframe.as_str(), e));
                    }
                }
            }
        }

        // Discover Polymarket up/down markets
        for config in configs.iter().filter(|c| c.platform != CryptoPlatform::KalshiOnly) {
            if let Some(pattern) = config.poly_slug_pattern {
                match self.discover_poly_updown(config.asset, pattern).await {
                    Ok(markets) => {
                        info!("  ₿ {} {}: {} updown markets found",
                              config.asset, config.timeframe.as_str(), markets.len());
                        result.poly_updown_markets.extend(markets);
                    }
                    Err(e) => {
                        result.errors.push(format!("Polymarket {} {}: {}",
                            config.asset.as_str(), config.timeframe.as_str(), e));
                    }
                }
            }
        }

        result.total_markets = result.kalshi_bracket_groups.iter().map(|g| g.brackets.len()).sum::<usize>()
            + result.poly_updown_markets.len();

        // Save to cache
        let cache = CryptoCache {
            timestamp_secs: current_unix_secs(),
            kalshi_bracket_groups: result.kalshi_bracket_groups.clone(),
            poly_updown_markets: result.poly_updown_markets.clone(),
        };
        if let Err(e) = Self::save_cache(&cache).await {
            warn!("Failed to save crypto cache: {}", e);
        }

        info!("₿ Crypto discovery complete: {} bracket groups, {} updown markets",
              result.kalshi_bracket_groups.len(), result.poly_updown_markets.len());

        result
    }

    /// Discover Kalshi price bracket markets for an asset
    async fn discover_kalshi_brackets(
        &self,
        asset: CryptoAsset,
        series: &str,
    ) -> Result<Vec<KalshiBracketGroup>> {
        // Rate limit and acquire permit
        {
            let _permit = self.kalshi_semaphore.acquire().await
                .map_err(|e| anyhow::anyhow!("semaphore closed: {}", e))?;
            self.kalshi_limiter.until_ready().await;
        }

        // Get events for this series
        let events = self.kalshi.get_events(series, 20).await?;
        let mut groups = Vec::new();

        for event in events {
            // Rate limit between requests
            {
                let _permit = self.kalshi_semaphore.acquire().await
                    .map_err(|e| anyhow::anyhow!("semaphore closed: {}", e))?;
                self.kalshi_limiter.until_ready().await;
            }

            // Get markets for this event
            let markets = match self.kalshi.get_markets(&event.event_ticker).await {
                Ok(m) => m,
                Err(e) => {
                    warn!("  ⚠️ Failed to get markets for {}: {}", event.event_ticker, e);
                    continue;
                }
            };

            if markets.is_empty() {
                continue;
            }

            // Parse brackets from markets
            let mut brackets: Vec<KalshiBracket> = markets
                .iter()
                .filter_map(|m| {
                    // Parse the price threshold from market title or floor_strike
                    let threshold = m.floor_strike.or_else(|| parse_price_from_title(&m.title))?;
                    Some(KalshiBracket {
                        ticker: m.ticker.clone().into(),
                        threshold,
                        yes_ask_cents: m.yes_ask.map(|p| p as u16),
                        no_ask_cents: m.no_ask.map(|p| p as u16),
                    })
                })
                .collect();

            // Sort by threshold
            brackets.sort_by(|a, b| a.threshold.partial_cmp(&b.threshold).unwrap_or(std::cmp::Ordering::Equal));

            if brackets.len() >= 2 {
                // Parse settlement time from first market
                let settlement_time_secs = markets.first()
                    .and_then(|m| m.expiration_time.as_ref())
                    .and_then(|t| parse_iso_timestamp(t));

                groups.push(KalshiBracketGroup {
                    asset: asset.as_str().to_string(),
                    event_ticker: event.event_ticker.into(),
                    description: event.title.into(),
                    settlement_time_secs,
                    brackets,
                });
            }
        }

        Ok(groups)
    }

    /// Discover Polymarket up/down markets for an asset
    async fn discover_poly_updown(
        &self,
        asset: CryptoAsset,
        slug_pattern: &str,
    ) -> Result<Vec<PolyUpDownMarket>> {
        let mut markets = Vec::new();

        // Generate slugs for current and upcoming time windows
        // For 15-min markets, we generate slugs for the next few hours
        let now_secs = current_unix_secs();

        // For 15-min markets: generate next 4 hours worth (16 intervals)
        let intervals = if slug_pattern.contains("15m") { 16 } else { 4 };
        let interval_secs = if slug_pattern.contains("15m") { 15 * 60 } else { 60 * 60 };

        for i in 0..intervals {
            let target_time = now_secs + (i * interval_secs);
            let slug = generate_crypto_slug(slug_pattern, asset, target_time);

            // Rate limit
            {
                let _permit = self.gamma_semaphore.acquire().await
                    .map_err(|e| anyhow::anyhow!("semaphore closed: {}", e))?;
                self.gamma_limiter.until_ready().await;
            }

            // Look up market
            match self.lookup_poly_market(&slug).await {
                Ok(Some((yes_token, no_token, description))) => {
                    markets.push(PolyUpDownMarket {
                        asset: asset.as_str().to_string(),
                        slug: slug.into(),
                        yes_token: yes_token.into(),
                        no_token: no_token.into(),
                        settlement_time_secs: Some(target_time + interval_secs),
                        description: description.into(),
                    });
                }
                Ok(None) => {
                    debug!("  No market found for slug: {}", slug);
                }
                Err(e) => {
                    debug!("  Error looking up {}: {}", slug, e);
                }
            }
        }

        Ok(markets)
    }

    /// Look up a Polymarket market by slug
    async fn lookup_poly_market(&self, slug: &str) -> Result<Option<(String, String, String)>> {
        let url = format!("{}/markets?slug={}", GAMMA_API_BASE, slug);

        let resp = self.http.get(&url).send().await?;

        if !resp.status().is_success() {
            return Ok(None);
        }

        let markets: Vec<GammaMarketResponse> = resp.json().await?;

        if markets.is_empty() {
            return Ok(None);
        }

        let market = &markets[0];

        // Check if active and not closed
        if market.closed == Some(true) || market.active == Some(false) {
            return Ok(None);
        }

        // Parse clobTokenIds JSON array
        let token_ids: Vec<String> = market.clob_token_ids
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();

        if token_ids.len() >= 2 {
            let description = market.question.clone().unwrap_or_else(|| slug.to_string());
            Ok(Some((token_ids[0].clone(), token_ids[1].clone(), description)))
        } else {
            Ok(None)
        }
    }

    /// Check for same-platform arbitrage in a Kalshi bracket group
    /// Returns the arbitrage opportunity if found
    pub fn check_kalshi_bracket_arb(group: &KalshiBracketGroup) -> Option<KalshiBracketArb> {
        // For a bracket group, arb exists if:
        // Sum of all YES prices < 100 (covers all outcomes)
        // Or sum of some subset of YES prices < appropriate value

        let total_yes_cents: u32 = group.brackets.iter()
            .filter_map(|b| b.yes_ask_cents)
            .map(|p| p as u32)
            .sum();

        // Account for Kalshi fees (approximately 7% on mid-price positions)
        let fee_estimate = (total_yes_cents as f64 * 0.07) as u32;
        let total_with_fees = total_yes_cents + fee_estimate;

        // Arb exists if we can buy all YES positions for less than 100 cents
        // (since exactly one bracket will win, we get $1 payout)
        if total_with_fees < 100 {
            let profit_cents = 100 - total_with_fees;
            return Some(KalshiBracketArb {
                event_ticker: group.event_ticker.clone(),
                brackets_to_buy: group.brackets.clone(),
                total_cost_cents: total_with_fees as u16,
                profit_cents: profit_cents as u16,
                profit_percent: (profit_cents as f64 / total_with_fees as f64) * 100.0,
            });
        }

        // Also check for adjacent bracket arbs (buy YES on one, NO on adjacent)
        // This is more complex and would require deeper analysis of bracket overlaps

        None
    }

    /// Check for same-platform arbitrage in a Polymarket up/down market
    /// Returns true if YES + NO < 100 cents (minus spread)
    pub fn check_poly_updown_arb(
        yes_ask_cents: u16,
        no_ask_cents: u16
    ) -> Option<PolyUpDownArb> {
        let total = yes_ask_cents as u32 + no_ask_cents as u32;

        // Polymarket has no explicit fees but has spread built into prices
        // Arb exists if total < 100
        if total < 100 {
            let profit_cents = 100 - total;
            return Some(PolyUpDownArb {
                yes_price_cents: yes_ask_cents,
                no_price_cents: no_ask_cents,
                profit_cents: profit_cents as u16,
                profit_percent: (profit_cents as f64 / total as f64) * 100.0,
            });
        }

        None
    }
}

/// A detected Kalshi bracket arbitrage opportunity
#[derive(Debug, Clone)]
pub struct KalshiBracketArb {
    /// Event ticker
    pub event_ticker: Arc<str>,
    /// Brackets to buy YES on
    pub brackets_to_buy: Vec<KalshiBracket>,
    /// Total cost in cents
    pub total_cost_cents: u16,
    /// Profit in cents
    pub profit_cents: u16,
    /// Profit percentage
    pub profit_percent: f64,
}

/// A detected Polymarket up/down arbitrage opportunity
#[derive(Debug, Clone)]
pub struct PolyUpDownArb {
    /// YES price in cents
    pub yes_price_cents: u16,
    /// NO price in cents
    pub no_price_cents: u16,
    /// Profit in cents
    pub profit_cents: u16,
    /// Profit percentage
    pub profit_percent: f64,
}

/// Gamma API response structure
#[derive(Debug, Deserialize)]
struct GammaMarketResponse {
    #[serde(rename = "clobTokenIds")]
    clob_token_ids: Option<String>,
    question: Option<String>,
    active: Option<bool>,
    closed: Option<bool>,
}

/// Parse a price threshold from a market title
/// e.g., "BTC above $95,000?" -> 95000.0
fn parse_price_from_title(title: &str) -> Option<f64> {
    // Look for price patterns like "$95,000" or "$95000" or "95000"
    // Manual parsing without regex

    let mut in_number = false;
    let mut number_start = 0;
    let chars: Vec<char> = title.chars().collect();

    for (i, &c) in chars.iter().enumerate() {
        if c.is_ascii_digit() {
            if !in_number {
                in_number = true;
                number_start = i;
            }
        } else if in_number && (c == ',' || c == '.') {
            // Continue if this is part of a number (comma separator or decimal)
            // Check if next char is a digit
            if i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
                continue;
            } else {
                // End of number
                let number_str: String = chars[number_start..i]
                    .iter()
                    .filter(|&&ch| ch.is_ascii_digit() || ch == '.')
                    .collect();
                if let Ok(price) = number_str.parse::<f64>() {
                    if price > 100.0 && price < 10_000_000.0 {
                        return Some(price);
                    }
                }
                in_number = false;
            }
        } else if in_number {
            // End of number
            let number_str: String = chars[number_start..i]
                .iter()
                .filter(|&&ch| ch.is_ascii_digit() || ch == '.')
                .collect();
            if let Ok(price) = number_str.parse::<f64>() {
                if price > 100.0 && price < 10_000_000.0 {
                    return Some(price);
                }
            }
            in_number = false;
        }
    }

    // Check if we ended while in a number
    if in_number {
        let number_str: String = chars[number_start..]
            .iter()
            .filter(|&&ch| ch.is_ascii_digit() || ch == '.')
            .collect();
        if let Ok(price) = number_str.parse::<f64>() {
            if price > 100.0 && price < 10_000_000.0 {
                return Some(price);
            }
        }
    }

    None
}

/// Generate a Polymarket slug from a pattern and timestamp
fn generate_crypto_slug(pattern: &str, asset: CryptoAsset, timestamp_secs: u64) -> String {
    // Round timestamp to nearest interval boundary
    let interval_secs = if pattern.contains("15m") { 15 * 60 } else { 60 * 60 };
    let rounded = (timestamp_secs / interval_secs) * interval_secs;

    // Format date for slug
    let datetime = chrono_format_utc(rounded);

    pattern
        .replace("{asset}", asset.as_str())
        .replace("{timestamp}", &rounded.to_string())
        .replace("{date}", &datetime)
}

/// Simple UTC timestamp formatting (without chrono dependency)
fn chrono_format_utc(secs: u64) -> String {
    // Convert Unix timestamp to date components
    // Days since epoch
    let days = secs / 86400;
    let remaining = secs % 86400;
    let hours = remaining / 3600;

    // Calculate year, month, day from days since epoch
    let (year, month, day) = days_to_ymd(days as i64);

    format!("{:04}-{:02}-{:02}-{:02}h", year, month, day, hours)
}

/// Convert days since Unix epoch to year/month/day
fn days_to_ymd(days: i64) -> (i32, u32, u32) {
    // Algorithm from Howard Hinnant
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    (y as i32, m as u32, d as u32)
}

/// Parse ISO 8601 timestamp to Unix seconds
fn parse_iso_timestamp(s: &str) -> Option<u64> {
    let cleaned = s.trim().replace("Z", "").replace("+00:00", "");
    let parts: Vec<&str> = cleaned.split('T').collect();
    if parts.len() != 2 {
        return None;
    }

    let date_parts: Vec<&str> = parts[0].split('-').collect();
    if date_parts.len() != 3 {
        return None;
    }

    let year: i32 = date_parts[0].parse().ok()?;
    let month: u32 = date_parts[1].parse().ok()?;
    let day: u32 = date_parts[2].parse().ok()?;

    let time_str = parts[1].split('.').next()?;
    let time_parts: Vec<&str> = time_str.split(':').collect();
    if time_parts.len() < 2 {
        return None;
    }

    let hour: u32 = time_parts[0].parse().ok()?;
    let minute: u32 = time_parts[1].parse().ok()?;
    let second: u32 = time_parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);

    let days_from_epoch = ymd_to_days(year, month, day)?;
    let seconds = (days_from_epoch as u64) * 86400 + (hour as u64) * 3600 + (minute as u64) * 60 + (second as u64);

    Some(seconds)
}

/// Convert year/month/day to days since Unix epoch
fn ymd_to_days(year: i32, month: u32, day: u32) -> Option<i64> {
    let year = year as i64;
    let month = month as i64;
    let day = day as i64;

    let (y, m) = if month <= 2 {
        (year - 1, month + 12)
    } else {
        (year, month)
    };

    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u64;
    let doy = (153 * (m - 3) as u64 + 2) / 5 + day as u64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;

    let days = era * 146097 + doe as i64 - 719468;
    Some(days)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_price_from_title() {
        assert_eq!(parse_price_from_title("BTC above $95,000?"), Some(95000.0));
        assert_eq!(parse_price_from_title("ETH above $3,500.50?"), Some(3500.50));
        assert_eq!(parse_price_from_title("Will BTC be above 100000?"), Some(100000.0));
    }

    #[test]
    fn test_check_poly_updown_arb() {
        // Arb exists: 45 + 50 = 95 < 100
        let arb = CryptoDiscoveryClient::check_poly_updown_arb(45, 50);
        assert!(arb.is_some());
        let arb = arb.unwrap();
        assert_eq!(arb.profit_cents, 5);

        // No arb: 50 + 52 = 102 > 100
        let no_arb = CryptoDiscoveryClient::check_poly_updown_arb(50, 52);
        assert!(no_arb.is_none());
    }

    #[test]
    fn test_days_to_ymd() {
        // 1970-01-01 = day 0
        assert_eq!(days_to_ymd(0), (1970, 1, 1));

        // 2000-01-01 = day 10957
        assert_eq!(days_to_ymd(10957), (2000, 1, 1));
    }
}
