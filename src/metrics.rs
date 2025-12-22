//! Metrics collection and comparison for trading performance analysis.
//!
//! This module tracks key performance indicators to compare the standard
//! execution mode vs priority-based execution mode.

use std::sync::atomic::{AtomicU64, AtomicI64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use tracing::info;

/// Atomic metrics counter for thread-safe accumulation
#[derive(Debug, Default)]
pub struct MetricsCollector {
    /// Total number of opportunities detected
    pub opportunities_detected: AtomicU64,
    /// Total number of trades executed
    pub trades_executed: AtomicU64,
    /// Total number of trades rejected (didn't meet criteria)
    pub trades_rejected: AtomicU64,
    /// Trades rejected due to low profit
    pub rejected_low_profit: AtomicU64,
    /// Trades rejected due to low liquidity
    pub rejected_low_liquidity: AtomicU64,
    /// Trades rejected due to high liquidity (clamped)
    pub trades_clamped: AtomicU64,
    /// Live game trades executed
    pub live_game_trades: AtomicU64,
    /// Total profit in cents (signed for losses)
    pub total_profit_cents: AtomicI64,
    /// Total volume traded in cents
    pub total_volume_cents: AtomicU64,
    /// Total Kalshi fees paid in cents
    pub total_fees_cents: AtomicU64,
    /// Fastest execution latency in nanoseconds
    pub min_latency_ns: AtomicU64,
    /// Slowest execution latency in nanoseconds
    pub max_latency_ns: AtomicU64,
    /// Sum of all latencies for averaging
    pub total_latency_ns: AtomicU64,
    /// Start time (Unix seconds)
    pub start_time_secs: AtomicU64,
    /// Priority mode enabled
    pub priority_mode: AtomicU64,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(priority_mode: bool) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let collector = Self::default();
        collector.start_time_secs.store(now, Ordering::SeqCst);
        collector.priority_mode.store(if priority_mode { 1 } else { 0 }, Ordering::SeqCst);
        collector.min_latency_ns.store(u64::MAX, Ordering::SeqCst);
        collector
    }

    /// Record an opportunity detection
    pub fn record_opportunity(&self) {
        self.opportunities_detected.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a successful trade execution
    pub fn record_trade(&self, profit_cents: i64, volume_cents: u64, fees_cents: u64, latency_ns: u64, is_live: bool) {
        self.trades_executed.fetch_add(1, Ordering::Relaxed);
        self.total_profit_cents.fetch_add(profit_cents, Ordering::Relaxed);
        self.total_volume_cents.fetch_add(volume_cents, Ordering::Relaxed);
        self.total_fees_cents.fetch_add(fees_cents, Ordering::Relaxed);
        self.total_latency_ns.fetch_add(latency_ns, Ordering::Relaxed);

        if is_live {
            self.live_game_trades.fetch_add(1, Ordering::Relaxed);
        }

        // Update min/max latency atomically
        let mut current_min = self.min_latency_ns.load(Ordering::Relaxed);
        while latency_ns < current_min {
            match self.min_latency_ns.compare_exchange_weak(
                current_min,
                latency_ns,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_min = x,
            }
        }

        let mut current_max = self.max_latency_ns.load(Ordering::Relaxed);
        while latency_ns > current_max {
            match self.max_latency_ns.compare_exchange_weak(
                current_max,
                latency_ns,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_max = x,
            }
        }
    }

    /// Record a rejected trade
    pub fn record_rejection(&self, reason: RejectionReason) {
        self.trades_rejected.fetch_add(1, Ordering::Relaxed);
        match reason {
            RejectionReason::LowProfit => {
                self.rejected_low_profit.fetch_add(1, Ordering::Relaxed);
            }
            RejectionReason::LowLiquidity => {
                self.rejected_low_liquidity.fetch_add(1, Ordering::Relaxed);
            }
            RejectionReason::Clamped => {
                self.trades_clamped.fetch_add(1, Ordering::Relaxed);
            }
            RejectionReason::Other => {}
        }
    }

    /// Get a snapshot of current metrics
    pub fn snapshot(&self) -> MetricsSnapshot {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let start = self.start_time_secs.load(Ordering::SeqCst);
        let duration_secs = now.saturating_sub(start);
        let trades = self.trades_executed.load(Ordering::Relaxed);

        MetricsSnapshot {
            priority_mode: self.priority_mode.load(Ordering::SeqCst) == 1,
            duration_secs,
            opportunities_detected: self.opportunities_detected.load(Ordering::Relaxed),
            trades_executed: trades,
            trades_rejected: self.trades_rejected.load(Ordering::Relaxed),
            rejected_low_profit: self.rejected_low_profit.load(Ordering::Relaxed),
            rejected_low_liquidity: self.rejected_low_liquidity.load(Ordering::Relaxed),
            trades_clamped: self.trades_clamped.load(Ordering::Relaxed),
            live_game_trades: self.live_game_trades.load(Ordering::Relaxed),
            total_profit_cents: self.total_profit_cents.load(Ordering::Relaxed),
            total_volume_cents: self.total_volume_cents.load(Ordering::Relaxed),
            total_fees_cents: self.total_fees_cents.load(Ordering::Relaxed),
            avg_latency_ns: if trades > 0 {
                self.total_latency_ns.load(Ordering::Relaxed) / trades
            } else {
                0
            },
            min_latency_ns: {
                let min = self.min_latency_ns.load(Ordering::Relaxed);
                if min == u64::MAX { 0 } else { min }
            },
            max_latency_ns: self.max_latency_ns.load(Ordering::Relaxed),
        }
    }

    /// Log current metrics summary
    pub fn log_summary(&self) {
        let snap = self.snapshot();
        let mode = if snap.priority_mode { "PRIORITY" } else { "STANDARD" };
        let hours = snap.duration_secs as f64 / 3600.0;

        info!("═══════════════════════════════════════════════════════════════");
        info!("📊 METRICS SUMMARY - {} MODE", mode);
        info!("═══════════════════════════════════════════════════════════════");
        info!("Duration: {:.2} hours", hours);
        info!("");
        info!("OPPORTUNITIES:");
        info!("  Detected: {}", snap.opportunities_detected);
        info!("  Executed: {} ({:.1}% conversion)",
              snap.trades_executed,
              if snap.opportunities_detected > 0 {
                  snap.trades_executed as f64 / snap.opportunities_detected as f64 * 100.0
              } else { 0.0 });
        info!("  Rejected: {}", snap.trades_rejected);
        info!("    - Low profit: {}", snap.rejected_low_profit);
        info!("    - Low liquidity: {}", snap.rejected_low_liquidity);
        info!("    - Clamped to max: {}", snap.trades_clamped);
        info!("");
        info!("PERFORMANCE:");
        info!("  Total profit: ${:.2}", snap.total_profit_cents as f64 / 100.0);
        info!("  Total volume: ${:.2}", snap.total_volume_cents as f64 / 100.0);
        info!("  Total fees: ${:.2}", snap.total_fees_cents as f64 / 100.0);
        info!("  Net profit: ${:.2}", (snap.total_profit_cents - snap.total_fees_cents as i64) as f64 / 100.0);
        info!("");
        info!("RATES (per hour):");
        if hours > 0.0 {
            info!("  Trades/hour: {:.1}", snap.trades_executed as f64 / hours);
            info!("  Profit/hour: ${:.2}", snap.total_profit_cents as f64 / 100.0 / hours);
            info!("  Volume/hour: ${:.2}", snap.total_volume_cents as f64 / 100.0 / hours);
        }
        info!("");
        info!("LATENCY:");
        info!("  Avg: {:.2}ms", snap.avg_latency_ns as f64 / 1_000_000.0);
        info!("  Min: {:.2}ms", snap.min_latency_ns as f64 / 1_000_000.0);
        info!("  Max: {:.2}ms", snap.max_latency_ns as f64 / 1_000_000.0);
        info!("");
        if snap.priority_mode {
            info!("PRIORITY MODE STATS:");
            info!("  Live game trades: {} ({:.1}% of total)",
                  snap.live_game_trades,
                  if snap.trades_executed > 0 {
                      snap.live_game_trades as f64 / snap.trades_executed as f64 * 100.0
                  } else { 0.0 });
        }
        info!("═══════════════════════════════════════════════════════════════");
    }
}

/// Reason for trade rejection
#[derive(Debug, Clone, Copy)]
pub enum RejectionReason {
    LowProfit,
    LowLiquidity,
    Clamped,
    Other,
}

/// Serializable snapshot of metrics at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub priority_mode: bool,
    pub duration_secs: u64,
    pub opportunities_detected: u64,
    pub trades_executed: u64,
    pub trades_rejected: u64,
    pub rejected_low_profit: u64,
    pub rejected_low_liquidity: u64,
    pub trades_clamped: u64,
    pub live_game_trades: u64,
    pub total_profit_cents: i64,
    pub total_volume_cents: u64,
    pub total_fees_cents: u64,
    pub avg_latency_ns: u64,
    pub min_latency_ns: u64,
    pub max_latency_ns: u64,
}

impl MetricsSnapshot {
    /// Calculate rate of return (profit / volume)
    pub fn rate_of_return(&self) -> f64 {
        if self.total_volume_cents > 0 {
            self.total_profit_cents as f64 / self.total_volume_cents as f64 * 100.0
        } else {
            0.0
        }
    }

    /// Calculate trades per hour
    pub fn trades_per_hour(&self) -> f64 {
        if self.duration_secs > 0 {
            self.trades_executed as f64 / (self.duration_secs as f64 / 3600.0)
        } else {
            0.0
        }
    }

    /// Calculate profit per hour
    pub fn profit_per_hour(&self) -> f64 {
        if self.duration_secs > 0 {
            (self.total_profit_cents as f64 / 100.0) / (self.duration_secs as f64 / 3600.0)
        } else {
            0.0
        }
    }

    /// Save snapshot to JSON file
    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, json)
    }

    /// Load snapshot from JSON file
    pub fn load(path: &str) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

/// Compare two metrics snapshots and print a comparison report
pub fn compare_snapshots(standard: &MetricsSnapshot, priority: &MetricsSnapshot) {
    info!("═══════════════════════════════════════════════════════════════════════════");
    info!("📊 COMPARISON: STANDARD vs PRIORITY MODE");
    info!("═══════════════════════════════════════════════════════════════════════════");
    info!("");
    info!("{:<30} {:>15} {:>15} {:>15}", "METRIC", "STANDARD", "PRIORITY", "DIFFERENCE");
    info!("{:-<30} {:-^15} {:-^15} {:-^15}", "", "", "", "");

    // Duration
    let std_hrs = standard.duration_secs as f64 / 3600.0;
    let pri_hrs = priority.duration_secs as f64 / 3600.0;
    info!("{:<30} {:>12.2}h {:>12.2}h {:>15}", "Duration", std_hrs, pri_hrs, "-");

    // Trades
    info!("{:<30} {:>15} {:>15} {:>+15}",
          "Trades Executed",
          standard.trades_executed,
          priority.trades_executed,
          priority.trades_executed as i64 - standard.trades_executed as i64);

    // Trades per hour
    let std_tph = standard.trades_per_hour();
    let pri_tph = priority.trades_per_hour();
    info!("{:<30} {:>12.1}/h {:>12.1}/h {:>+12.1}/h",
          "Trade Rate",
          std_tph, pri_tph, pri_tph - std_tph);

    // Profit
    let std_profit = standard.total_profit_cents as f64 / 100.0;
    let pri_profit = priority.total_profit_cents as f64 / 100.0;
    info!("{:<30} {:>13.2}$ {:>13.2}$ {:>+13.2}$",
          "Total Profit",
          std_profit, pri_profit, pri_profit - std_profit);

    // Profit per hour
    let std_pph = standard.profit_per_hour();
    let pri_pph = priority.profit_per_hour();
    info!("{:<30} {:>11.2}$/h {:>11.2}$/h {:>+11.2}$/h",
          "Profit Rate",
          std_pph, pri_pph, pri_pph - std_pph);

    // Volume
    let std_vol = standard.total_volume_cents as f64 / 100.0;
    let pri_vol = priority.total_volume_cents as f64 / 100.0;
    info!("{:<30} {:>13.2}$ {:>13.2}$ {:>+13.2}$",
          "Total Volume",
          std_vol, pri_vol, pri_vol - std_vol);

    // Rate of return
    let std_ror = standard.rate_of_return();
    let pri_ror = priority.rate_of_return();
    info!("{:<30} {:>13.2}% {:>13.2}% {:>+13.2}%",
          "Rate of Return",
          std_ror, pri_ror, pri_ror - std_ror);

    // Fees
    let std_fees = standard.total_fees_cents as f64 / 100.0;
    let pri_fees = priority.total_fees_cents as f64 / 100.0;
    info!("{:<30} {:>13.2}$ {:>13.2}$ {:>+13.2}$",
          "Total Fees",
          std_fees, pri_fees, pri_fees - std_fees);

    // Rejections
    info!("{:<30} {:>15} {:>15} {:>+15}",
          "Trades Rejected",
          standard.trades_rejected,
          priority.trades_rejected,
          priority.trades_rejected as i64 - standard.trades_rejected as i64);

    // Latency
    let std_lat = standard.avg_latency_ns as f64 / 1_000_000.0;
    let pri_lat = priority.avg_latency_ns as f64 / 1_000_000.0;
    info!("{:<30} {:>12.2}ms {:>12.2}ms {:>+12.2}ms",
          "Avg Latency",
          std_lat, pri_lat, pri_lat - std_lat);

    // Live game trades (priority only)
    info!("{:<30} {:>15} {:>15} {:>15}",
          "Live Game Trades",
          "-",
          priority.live_game_trades,
          "(priority only)");

    info!("");
    info!("═══════════════════════════════════════════════════════════════════════════");

    // Summary
    let profit_improvement = if std_profit > 0.0 {
        ((pri_profit - std_profit) / std_profit * 100.0)
    } else {
        0.0
    };

    let rate_improvement = if std_tph > 0.0 {
        ((pri_tph - std_tph) / std_tph * 100.0)
    } else {
        0.0
    };

    info!("");
    info!("SUMMARY:");
    info!("  Profit improvement: {:+.1}%", profit_improvement);
    info!("  Trade rate improvement: {:+.1}%", rate_improvement);
    info!("  Priority mode {} more profitable",
          if pri_profit > std_profit { "IS" } else { "is NOT" });
    info!("");
}

/// Global metrics instance
static METRICS: std::sync::OnceLock<Arc<MetricsCollector>> = std::sync::OnceLock::new();

/// Initialize global metrics collector
pub fn init_metrics(priority_mode: bool) -> Arc<MetricsCollector> {
    METRICS.get_or_init(|| Arc::new(MetricsCollector::new(priority_mode))).clone()
}

/// Get the global metrics collector
pub fn get_metrics() -> Option<Arc<MetricsCollector>> {
    METRICS.get().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collection() {
        let metrics = MetricsCollector::new(true);

        metrics.record_opportunity();
        metrics.record_opportunity();
        metrics.record_trade(100, 1000, 5, 1_000_000, false);
        metrics.record_trade(50, 500, 3, 2_000_000, true);
        metrics.record_rejection(RejectionReason::LowProfit);

        let snap = metrics.snapshot();
        assert_eq!(snap.opportunities_detected, 2);
        assert_eq!(snap.trades_executed, 2);
        assert_eq!(snap.trades_rejected, 1);
        assert_eq!(snap.total_profit_cents, 150);
        assert_eq!(snap.live_game_trades, 1);
    }

    #[test]
    fn test_rate_calculations() {
        let snap = MetricsSnapshot {
            priority_mode: false,
            duration_secs: 3600,  // 1 hour
            opportunities_detected: 100,
            trades_executed: 10,
            trades_rejected: 5,
            rejected_low_profit: 3,
            rejected_low_liquidity: 2,
            trades_clamped: 0,
            live_game_trades: 0,
            total_profit_cents: 1000,  // $10
            total_volume_cents: 10000, // $100
            total_fees_cents: 50,
            avg_latency_ns: 1_000_000,
            min_latency_ns: 500_000,
            max_latency_ns: 2_000_000,
        };

        assert_eq!(snap.trades_per_hour(), 10.0);
        assert_eq!(snap.profit_per_hour(), 10.0);
        assert_eq!(snap.rate_of_return(), 10.0);
    }
}
