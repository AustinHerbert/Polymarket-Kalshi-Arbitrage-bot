//! Background balance tracker for dynamic position sizing.
//!
//! Fetches account balances periodically in the background and provides
//! fast cached lookups for the execution engine.

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{info, warn, debug};

use crate::kalshi::KalshiApiClient;

/// Tracks account balances with background refresh
pub struct BalanceTracker {
    /// Cached Kalshi balance in cents (atomic for lock-free reads)
    kalshi_balance_cents: AtomicI64,
    /// Whether balance has been successfully fetched at least once
    initialized: std::sync::atomic::AtomicBool,
}

impl BalanceTracker {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            kalshi_balance_cents: AtomicI64::new(0),
            initialized: std::sync::atomic::AtomicBool::new(false),
        })
    }

    /// Get cached Kalshi balance in cents (lock-free)
    #[inline]
    pub fn kalshi_balance_cents(&self) -> Option<i64> {
        if self.initialized.load(Ordering::Relaxed) {
            Some(self.kalshi_balance_cents.load(Ordering::Relaxed))
        } else {
            None
        }
    }

    /// Update Kalshi balance (called by background task)
    fn update_kalshi(&self, balance_cents: i64) {
        self.kalshi_balance_cents.store(balance_cents, Ordering::Relaxed);
        self.initialized.store(true, Ordering::Relaxed);
    }

    /// Calculate max position size based on available balance
    /// Returns size in cents, capped by config max if provided
    pub fn max_position_cents(&self, config_max_cents: u32) -> u32 {
        match self.kalshi_balance_cents() {
            Some(balance) => {
                // Use 90% of balance to leave buffer for fees
                let safe_balance = ((balance as f64) * 0.9) as u32;
                safe_balance.min(config_max_cents)
            }
            None => config_max_cents, // Fall back to config if no balance yet
        }
    }
}

/// Background task to periodically refresh balances
pub async fn run_balance_tracker(
    tracker: Arc<BalanceTracker>,
    kalshi: Arc<KalshiApiClient>,
    refresh_interval_secs: u64,
) {
    let mut ticker = interval(Duration::from_secs(refresh_interval_secs));

    info!("[BALANCE] Starting balance tracker (refresh every {}s)", refresh_interval_secs);

    // Fetch immediately on start
    fetch_and_update(&tracker, &kalshi).await;

    loop {
        ticker.tick().await;
        fetch_and_update(&tracker, &kalshi).await;
    }
}

async fn fetch_and_update(tracker: &BalanceTracker, kalshi: &KalshiApiClient) {
    match kalshi.get_balance().await {
        Ok(balance_cents) => {
            let old = tracker.kalshi_balance_cents.load(Ordering::Relaxed);
            tracker.update_kalshi(balance_cents);

            if old != balance_cents {
                info!("[BALANCE] Kalshi: ${:.2}", balance_cents as f64 / 100.0);
            } else {
                debug!("[BALANCE] Kalshi: ${:.2} (unchanged)", balance_cents as f64 / 100.0);
            }
        }
        Err(e) => {
            warn!("[BALANCE] Failed to fetch Kalshi balance: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balance_tracker_initial_state() {
        let tracker = BalanceTracker::new();
        assert!(tracker.kalshi_balance_cents().is_none());
        assert_eq!(tracker.max_position_cents(10000), 10000); // Falls back to config
    }

    #[test]
    fn test_balance_tracker_with_balance() {
        let tracker = BalanceTracker::new();
        tracker.update_kalshi(100000); // $1000

        assert_eq!(tracker.kalshi_balance_cents(), Some(100000));
        // 90% of $1000 = $900 = 90000 cents
        assert_eq!(tracker.max_position_cents(200000), 90000);
        // But capped at config max if lower
        assert_eq!(tracker.max_position_cents(50000), 50000);
    }
}
