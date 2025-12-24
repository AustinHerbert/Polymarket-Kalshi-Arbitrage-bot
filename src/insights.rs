//! Bot insights and activity tracking for dashboard visibility.
//!
//! Tracks near-miss opportunities, market activity, and provides
//! real-time feedback on what the bot is seeing and doing.

use chrono::{DateTime, Utc, Timelike};
use parking_lot::RwLock;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Global insights tracker
static INSIGHTS: std::sync::OnceLock<InsightsTracker> = std::sync::OnceLock::new();

/// Initialize the global insights tracker
pub fn init_insights() -> &'static InsightsTracker {
    INSIGHTS.get_or_init(InsightsTracker::new)
}

/// Get the global insights tracker
pub fn get_insights() -> Option<&'static InsightsTracker> {
    INSIGHTS.get()
}

/// A near-miss opportunity that almost qualified as an arb
#[derive(Debug, Clone, Serialize)]
pub struct NearMiss {
    pub timestamp: DateTime<Utc>,
    pub market: String,
    pub league: String,
    /// Total cost in cents (yes + no price)
    pub total_cost_cents: u16,
    /// How many cents away from being profitable (e.g., 3 = need 3¢ cheaper)
    pub gap_cents: u16,
    /// Gap as percentage
    pub gap_percent: f64,
    /// Available liquidity in dollars
    pub liquidity_dollars: f64,
    /// Which side was more expensive
    pub expensive_side: String,
}

/// Hourly activity summary
#[derive(Debug, Clone, Serialize, Default)]
pub struct HourlyStats {
    pub hour: u32,
    pub scans: u64,
    pub near_misses: u64,
    pub executions: u64,
    pub best_gap_cents: u16,
    pub avg_gap_cents: f64,
}

/// Per-league statistics
#[derive(Debug, Clone, Serialize, Default)]
pub struct LeagueStats {
    pub league: String,
    pub near_misses: u64,
    pub executions: u64,
    pub avg_liquidity_dollars: f64,
    pub best_gap_cents: u16,
    pub markets_tracked: u32,
}

/// Current market spread for live view
#[derive(Debug, Clone, Serialize)]
pub struct MarketSpread {
    pub market: String,
    pub league: String,
    pub yes_price: u16,
    pub no_price: u16,
    pub total_cost: u16,
    pub gap_cents: i16,  // Negative = profitable arb
    pub liquidity_dollars: f64,
    pub last_update: DateTime<Utc>,
}

/// Full insights snapshot for API
#[derive(Debug, Clone, Serialize)]
pub struct InsightsSnapshot {
    pub generated_at: DateTime<Utc>,
    pub uptime_hours: f64,
    pub total_scans: u64,
    pub total_near_misses: u64,
    pub total_executions: u64,
    /// Recent near misses (last 50)
    pub recent_near_misses: Vec<NearMiss>,
    /// Hourly breakdown (last 24 hours)
    pub hourly_stats: Vec<HourlyStats>,
    /// Per-league stats
    pub league_stats: Vec<LeagueStats>,
    /// Current best spreads (top 10 closest to arb)
    pub current_spreads: Vec<MarketSpread>,
    /// Insight messages
    pub insights: Vec<String>,
}

/// Main insights tracker
pub struct InsightsTracker {
    start_time: DateTime<Utc>,
    /// Atomic counters for high-frequency updates
    total_scans: AtomicU64,
    total_near_misses: AtomicU64,
    total_executions: AtomicU64,
    /// Recent near misses (circular buffer, keeps last 100)
    near_misses: RwLock<Vec<NearMiss>>,
    /// Hourly stats (24 hours)
    hourly: RwLock<[HourlyStats; 24]>,
    /// Per-league stats
    leagues: RwLock<HashMap<String, LeagueStats>>,
    /// Current market spreads
    spreads: RwLock<HashMap<String, MarketSpread>>,
}

impl InsightsTracker {
    pub fn new() -> Self {
        Self {
            start_time: Utc::now(),
            total_scans: AtomicU64::new(0),
            total_near_misses: AtomicU64::new(0),
            total_executions: AtomicU64::new(0),
            near_misses: RwLock::new(Vec::with_capacity(100)),
            hourly: RwLock::new(std::array::from_fn(|i| HourlyStats { hour: i as u32, ..Default::default() })),
            leagues: RwLock::new(HashMap::new()),
            spreads: RwLock::new(HashMap::new()),
        }
    }

    /// Record a market scan (called frequently)
    #[inline]
    pub fn record_scan(&self) {
        self.total_scans.fetch_add(1, Ordering::Relaxed);
        let hour = Utc::now().hour() as usize;
        self.hourly.write()[hour].scans += 1;
    }

    /// Record a near-miss opportunity
    pub fn record_near_miss(
        &self,
        market: &str,
        league: &str,
        total_cost_cents: u16,
        liquidity_cents: u32,
        expensive_side: &str,
    ) {
        let gap_cents = total_cost_cents.saturating_sub(100);
        if gap_cents > 10 {
            return; // Only track if within 10 cents of arb
        }

        let gap_percent = (gap_cents as f64 / 100.0) * 100.0;
        let liquidity_dollars = liquidity_cents as f64 / 100.0;

        let near_miss = NearMiss {
            timestamp: Utc::now(),
            market: market.to_string(),
            league: league.to_string(),
            total_cost_cents,
            gap_cents,
            gap_percent,
            liquidity_dollars,
            expensive_side: expensive_side.to_string(),
        };

        // Update counters
        self.total_near_misses.fetch_add(1, Ordering::Relaxed);
        let hour = Utc::now().hour() as usize;
        {
            let mut hourly = self.hourly.write();
            hourly[hour].near_misses += 1;
            if gap_cents < hourly[hour].best_gap_cents || hourly[hour].best_gap_cents == 0 {
                hourly[hour].best_gap_cents = gap_cents;
            }
        }

        // Update league stats
        {
            let mut leagues = self.leagues.write();
            let stats = leagues.entry(league.to_string()).or_insert_with(|| LeagueStats {
                league: league.to_string(),
                ..Default::default()
            });
            stats.near_misses += 1;
            if gap_cents < stats.best_gap_cents || stats.best_gap_cents == 0 {
                stats.best_gap_cents = gap_cents;
            }
        }

        // Add to recent near misses (keep last 100)
        {
            let mut misses = self.near_misses.write();
            if misses.len() >= 100 {
                misses.remove(0);
            }
            misses.push(near_miss);
        }
    }

    /// Record an execution
    pub fn record_execution(&self, league: &str) {
        self.total_executions.fetch_add(1, Ordering::Relaxed);
        let hour = Utc::now().hour() as usize;
        self.hourly.write()[hour].executions += 1;

        let mut leagues = self.leagues.write();
        let stats = leagues.entry(league.to_string()).or_insert_with(|| LeagueStats {
            league: league.to_string(),
            ..Default::default()
        });
        stats.executions += 1;
    }

    /// Update current market spread (called on price updates)
    pub fn update_spread(
        &self,
        market_id: &str,
        market: &str,
        league: &str,
        yes_price: u16,
        no_price: u16,
        liquidity_cents: u32,
    ) {
        let total_cost = yes_price + no_price;
        let gap_cents = total_cost as i16 - 100;
        let liquidity_dollars = liquidity_cents as f64 / 100.0;

        let spread = MarketSpread {
            market: market.to_string(),
            league: league.to_string(),
            yes_price,
            no_price,
            total_cost,
            gap_cents,
            liquidity_dollars,
            last_update: Utc::now(),
        };

        self.spreads.write().insert(market_id.to_string(), spread);
    }

    /// Update league market count
    pub fn set_league_markets(&self, league: &str, count: u32) {
        let mut leagues = self.leagues.write();
        let stats = leagues.entry(league.to_string()).or_insert_with(|| LeagueStats {
            league: league.to_string(),
            ..Default::default()
        });
        stats.markets_tracked = count;
    }

    /// Get full insights snapshot
    pub fn snapshot(&self) -> InsightsSnapshot {
        let now = Utc::now();
        let uptime = now.signed_duration_since(self.start_time);
        let uptime_hours = uptime.num_seconds() as f64 / 3600.0;

        // Get recent near misses (last 50)
        let recent_near_misses: Vec<NearMiss> = {
            let misses = self.near_misses.read();
            misses.iter().rev().take(50).cloned().collect()
        };

        // Get hourly stats
        let hourly_stats: Vec<HourlyStats> = {
            let hourly = self.hourly.read();
            hourly.iter().cloned().collect()
        };

        // Get league stats sorted by near misses
        let mut league_stats: Vec<LeagueStats> = {
            let leagues = self.leagues.read();
            leagues.values().cloned().collect()
        };
        league_stats.sort_by(|a, b| b.near_misses.cmp(&a.near_misses));

        // Get current spreads sorted by gap (closest to arb first)
        let mut current_spreads: Vec<MarketSpread> = {
            let spreads = self.spreads.read();
            spreads.values()
                .filter(|s| s.gap_cents <= 10 && s.gap_cents >= -5) // Only show close ones
                .cloned()
                .collect()
        };
        current_spreads.sort_by_key(|s| s.gap_cents);
        current_spreads.truncate(10);

        // Generate insight messages
        let insights = self.generate_insights(&recent_near_misses, &league_stats, &current_spreads);

        InsightsSnapshot {
            generated_at: now,
            uptime_hours,
            total_scans: self.total_scans.load(Ordering::Relaxed),
            total_near_misses: self.total_near_misses.load(Ordering::Relaxed),
            total_executions: self.total_executions.load(Ordering::Relaxed),
            recent_near_misses,
            hourly_stats,
            league_stats,
            current_spreads,
            insights,
        }
    }

    /// Generate human-readable insights
    fn generate_insights(
        &self,
        near_misses: &[NearMiss],
        leagues: &[LeagueStats],
        spreads: &[MarketSpread],
    ) -> Vec<String> {
        let mut insights = Vec::new();

        // Best current opportunity
        if let Some(best) = spreads.first() {
            if best.gap_cents <= 0 {
                insights.push(format!(
                    "🎯 Live arb available: {} ({}¢ profit)",
                    best.market, -best.gap_cents
                ));
            } else if best.gap_cents <= 3 {
                insights.push(format!(
                    "👀 Very close: {} just {}¢ away from arb",
                    best.market, best.gap_cents
                ));
            }
        }

        // Most active league
        if let Some(top_league) = leagues.first() {
            if top_league.near_misses > 0 {
                insights.push(format!(
                    "📊 {} has most activity: {} near-misses, {} executions",
                    top_league.league, top_league.near_misses, top_league.executions
                ));
            }
        }

        // Recent pattern
        if !near_misses.is_empty() {
            let last_hour: Vec<_> = near_misses.iter()
                .filter(|m| (Utc::now() - m.timestamp).num_minutes() < 60)
                .collect();

            if last_hour.is_empty() {
                insights.push("💤 No near-misses in the last hour - markets are quiet".to_string());
            } else {
                let avg_gap: f64 = last_hour.iter().map(|m| m.gap_cents as f64).sum::<f64>()
                    / last_hour.len() as f64;
                insights.push(format!(
                    "⚡ {} near-misses in last hour (avg gap: {:.1}¢)",
                    last_hour.len(), avg_gap
                ));
            }
        } else {
            insights.push("🔍 Scanning markets... no near-misses recorded yet".to_string());
        }

        // Liquidity insight
        if let Some(best_liq) = spreads.iter().max_by(|a, b|
            a.liquidity_dollars.partial_cmp(&b.liquidity_dollars).unwrap()
        ) {
            if best_liq.liquidity_dollars > 500.0 {
                insights.push(format!(
                    "💰 Best liquidity: {} (${:.0})",
                    best_liq.market, best_liq.liquidity_dollars
                ));
            }
        }

        insights
    }
}

impl Default for InsightsTracker {
    fn default() -> Self {
        Self::new()
    }
}
