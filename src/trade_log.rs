//! Trade logging for dry run tracking and dashboard display.
//!
//! Persists all trade attempts (both successful and rejected) to a JSON file
//! for dashboard visualization and performance analysis.

use std::fs::{self, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use parking_lot::Mutex;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// A single trade record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    pub id: u64,
    pub timestamp: DateTime<Utc>,
    pub market_id: u16,
    pub market_name: String,
    pub arb_type: String,
    pub yes_price_cents: u16,
    pub no_price_cents: u16,
    pub contracts: i64,
    pub profit_cents: i64,
    pub volume_cents: u64,
    pub fees_cents: u64,
    pub latency_ms: f64,
    pub status: TradeStatus,
    pub rejection_reason: Option<String>,
    pub is_dry_run: bool,
    /// When the event expires/settles (Unix timestamp seconds)
    /// Profit is only "realized" after this time
    #[serde(default)]
    pub event_expires_at: Option<u64>,
    /// Whether the event has settled (expires_at has passed)
    #[serde(default)]
    pub is_settled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TradeStatus {
    Executed,
    Rejected,
    DryRun,
}

/// Summary statistics for the dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeSummary {
    pub start_time: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
    pub total_trades: u64,
    pub successful_trades: u64,
    pub rejected_trades: u64,
    pub total_profit_cents: i64,
    pub total_volume_cents: u64,
    pub total_fees_cents: u64,
    pub avg_profit_per_trade_cents: f64,
    pub trades_per_hour: f64,
    pub profit_per_hour_cents: f64,
    pub best_trade_profit_cents: i64,
    pub worst_trade_profit_cents: i64,
    pub avg_latency_ms: f64,
    pub is_dry_run: bool,
    pub bankroll_cents: u64,
}

impl TradeSummary {
    /// Project 30-day compounding returns
    pub fn project_30_days(&self) -> ProjectedReturns {
        let hours_running = (self.last_update - self.start_time).num_seconds() as f64 / 3600.0;
        if hours_running < 0.1 || self.total_volume_cents == 0 {
            return ProjectedReturns::default();
        }

        // Calculate hourly return rate
        let hourly_profit = self.profit_per_hour_cents;
        let hourly_return_rate = if self.bankroll_cents > 0 {
            hourly_profit / self.bankroll_cents as f64
        } else {
            0.0
        };

        // Project for 30 days (720 hours)
        let hours_per_day = 24.0;
        let days = 30;

        let mut daily_profits = Vec::new();
        let mut cumulative_bankroll = self.bankroll_cents as f64;

        for day in 1..=days {
            // Compound daily (assume 8 hours of active trading per day for realistic estimate)
            let trading_hours = 8.0;
            let daily_profit = cumulative_bankroll * hourly_return_rate * trading_hours;
            cumulative_bankroll += daily_profit;
            daily_profits.push(DailyProjection {
                day,
                profit_cents: daily_profit as i64,
                cumulative_profit_cents: (cumulative_bankroll - self.bankroll_cents as f64) as i64,
                bankroll_cents: cumulative_bankroll as u64,
            });
        }

        ProjectedReturns {
            hourly_return_rate: hourly_return_rate * 100.0,
            daily_return_rate: hourly_return_rate * hours_per_day * 100.0,
            projected_30_day_profit_cents: (cumulative_bankroll - self.bankroll_cents as f64) as i64,
            projected_30_day_bankroll_cents: cumulative_bankroll as u64,
            daily_projections: daily_profits,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectedReturns {
    pub hourly_return_rate: f64,
    pub daily_return_rate: f64,
    pub projected_30_day_profit_cents: i64,
    pub projected_30_day_bankroll_cents: u64,
    pub daily_projections: Vec<DailyProjection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyProjection {
    pub day: i32,
    pub profit_cents: i64,
    pub cumulative_profit_cents: i64,
    pub bankroll_cents: u64,
}

/// Trade logger that persists to JSON files
/// Uses parking_lot::Mutex for faster synchronous locking
pub struct TradeLogger {
    trades_file: String,
    summary_file: String,
    trades: Mutex<Vec<TradeRecord>>,
    /// Atomic counter for trade IDs - avoids lock contention
    next_id: AtomicU64,
    start_time: DateTime<Utc>,
    is_dry_run: bool,
    bankroll_cents: u64,
}

impl TradeLogger {
    pub fn new(base_dir: &str, is_dry_run: bool, bankroll_cents: u64) -> Self {
        let trades_file = format!("{}/trades.json", base_dir);
        let summary_file = format!("{}/summary.json", base_dir);

        // Create directory if needed
        if let Some(parent) = Path::new(&trades_file).parent() {
            let _ = fs::create_dir_all(parent);
        }

        // Load existing trades or start fresh
        let (trades, next_id) = if Path::new(&trades_file).exists() {
            match fs::read_to_string(&trades_file) {
                Ok(content) => {
                    match serde_json::from_str::<Vec<TradeRecord>>(&content) {
                        Ok(t) => {
                            let max_id = t.iter().map(|r| r.id).max().unwrap_or(0);
                            (t, max_id + 1)
                        }
                        Err(_) => (Vec::new(), 1)
                    }
                }
                Err(_) => (Vec::new(), 1)
            }
        } else {
            (Vec::new(), 1)
        };

        Self {
            trades_file,
            summary_file,
            trades: Mutex::new(trades),
            next_id: AtomicU64::new(next_id),
            start_time: Utc::now(),
            is_dry_run,
            bankroll_cents,
        }
    }

    /// Log a trade (executed or rejected)
    /// Uses atomic fetch_add for lock-free ID generation
    #[inline]
    pub fn log_trade(
        &self,
        market_id: u16,
        market_name: &str,
        arb_type: &str,
        yes_price_cents: u16,
        no_price_cents: u16,
        contracts: i64,
        profit_cents: i64,
        volume_cents: u64,
        fees_cents: u64,
        latency_ns: u64,
        status: TradeStatus,
        rejection_reason: Option<&str>,
    ) {
        self.log_trade_with_expiry(
            market_id, market_name, arb_type, yes_price_cents, no_price_cents,
            contracts, profit_cents, volume_cents, fees_cents, latency_ns,
            status, rejection_reason, None
        );
    }

    /// Log a trade with event expiration time for proper profit timing
    #[inline]
    pub fn log_trade_with_expiry(
        &self,
        market_id: u16,
        market_name: &str,
        arb_type: &str,
        yes_price_cents: u16,
        no_price_cents: u16,
        contracts: i64,
        profit_cents: i64,
        volume_cents: u64,
        fees_cents: u64,
        latency_ns: u64,
        status: TradeStatus,
        rejection_reason: Option<&str>,
        event_expires_at: Option<u64>,
    ) {
        // Lock-free ID generation with atomic increment
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);

        // Check if event has already settled
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let is_settled = event_expires_at.map(|exp| now_secs >= exp).unwrap_or(false);

        let record = TradeRecord {
            id,
            timestamp: Utc::now(),
            market_id,
            market_name: market_name.to_string(),
            arb_type: arb_type.to_string(),
            yes_price_cents,
            no_price_cents,
            contracts,
            profit_cents,
            volume_cents,
            fees_cents,
            latency_ms: latency_ns as f64 / 1_000_000.0,
            status,
            rejection_reason: rejection_reason.map(String::from),
            is_dry_run: self.is_dry_run,
            event_expires_at,
            is_settled,
        };

        // Add to in-memory list - parking_lot Mutex is faster
        self.trades.lock().push(record);

        // Persist to file
        self.save();
    }

    /// Save trades and summary to disk
    fn save(&self) {
        let mut trades = self.trades.lock();

        // Update is_settled status for all trades based on current time
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        for trade in trades.iter_mut() {
            if let Some(exp) = trade.event_expires_at {
                trade.is_settled = now_secs >= exp;
            }
        }

        // Save trades
        if let Ok(file) = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.trades_file)
        {
            let mut writer = BufWriter::new(file);
            let _ = serde_json::to_writer_pretty(&mut writer, &*trades);
            let _ = writer.flush();
        }

        // Calculate and save summary
        let summary = self.calculate_summary(&trades);
        if let Ok(file) = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.summary_file)
        {
            let mut writer = BufWriter::new(file);
            let _ = serde_json::to_writer_pretty(&mut writer, &summary);
            let _ = writer.flush();
        }
    }

    fn calculate_summary(&self, trades: &[TradeRecord]) -> TradeSummary {
        let now = Utc::now();
        let successful: Vec<_> = trades.iter()
            .filter(|t| t.status == TradeStatus::Executed || t.status == TradeStatus::DryRun)
            .collect();

        let total_profit: i64 = successful.iter().map(|t| t.profit_cents).sum();
        let total_volume: u64 = successful.iter().map(|t| t.volume_cents).sum();
        let total_fees: u64 = successful.iter().map(|t| t.fees_cents).sum();
        let total_latency: f64 = successful.iter().map(|t| t.latency_ms).sum();

        let hours_running = (now - self.start_time).num_seconds() as f64 / 3600.0;
        let hours = if hours_running > 0.0 { hours_running } else { 1.0 };

        TradeSummary {
            start_time: self.start_time,
            last_update: now,
            total_trades: trades.len() as u64,
            successful_trades: successful.len() as u64,
            rejected_trades: trades.iter().filter(|t| t.status == TradeStatus::Rejected).count() as u64,
            total_profit_cents: total_profit,
            total_volume_cents: total_volume,
            total_fees_cents: total_fees,
            avg_profit_per_trade_cents: if !successful.is_empty() {
                total_profit as f64 / successful.len() as f64
            } else { 0.0 },
            trades_per_hour: successful.len() as f64 / hours,
            profit_per_hour_cents: total_profit as f64 / hours,
            best_trade_profit_cents: successful.iter().map(|t| t.profit_cents).max().unwrap_or(0),
            worst_trade_profit_cents: successful.iter().map(|t| t.profit_cents).min().unwrap_or(0),
            avg_latency_ms: if !successful.is_empty() {
                total_latency / successful.len() as f64
            } else { 0.0 },
            is_dry_run: self.is_dry_run,
            bankroll_cents: self.bankroll_cents,
        }
    }

    /// Get current summary
    pub fn get_summary(&self) -> TradeSummary {
        let trades = self.trades.lock();
        self.calculate_summary(&trades)
    }

    /// Get all trades
    pub fn get_trades(&self) -> Vec<TradeRecord> {
        self.trades.lock().clone()
    }

    /// Clear all trades (for testing)
    pub fn clear(&self) {
        self.trades.lock().clear();
        self.next_id.store(1, Ordering::Relaxed);
        self.save();
    }
}

// Global trade logger
static TRADE_LOGGER: std::sync::OnceLock<TradeLogger> = std::sync::OnceLock::new();

/// Initialize the global trade logger
pub fn init_trade_logger(base_dir: &str, is_dry_run: bool, bankroll_cents: u64) -> &'static TradeLogger {
    TRADE_LOGGER.get_or_init(|| TradeLogger::new(base_dir, is_dry_run, bankroll_cents))
}

/// Get the global trade logger
pub fn get_trade_logger() -> Option<&'static TradeLogger> {
    TRADE_LOGGER.get()
}
