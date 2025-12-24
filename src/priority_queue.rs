//! Priority-based execution queue for arbitrage opportunities.
//!
//! This module provides a priority queue that sorts arbitrage opportunities based on:
//! 1. Live games (highest priority)
//! 2. Earliest expiration time (for faster capital turnover)
//! 3. Greatest profit percentage
//!
//! The queue re-sorts periodically (configurable) and applies liquidity constraints.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::time::{SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;
use tracing::{debug, info};

use crate::priority_config::PriorityConfig;
use crate::types::{FastExecutionRequest, GlobalState, MarketPair, kalshi_fee_cents};

/// Current Unix timestamp in seconds
fn current_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// A prioritized arbitrage opportunity with computed scores
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PrioritizedOpportunity {
    /// The underlying execution request
    pub request: FastExecutionRequest,
    /// Reference to the market pair for metadata
    pub pair: Arc<MarketPair>,
    /// Computed priority score (higher = more urgent)
    pub priority_score: f64,
    /// Profit percentage for this opportunity
    pub profit_percent: f64,
    /// Time until expiration in seconds (lower = more urgent)
    pub seconds_until_expiry: Option<u64>,
    /// Whether this is a live game
    pub is_live: bool,
    /// Timestamp when this opportunity was queued
    pub queued_at_secs: u64,
}

impl PrioritizedOpportunity {
    /// Create a new prioritized opportunity from an execution request
    pub fn new(
        request: FastExecutionRequest,
        pair: Arc<MarketPair>,
        config: &PriorityConfig,
    ) -> Self {
        let now = current_unix_secs();

        // Calculate profit percentage
        let profit_cents = request.profit_cents();
        let profit_percent = profit_cents as f64;  // Already in percent (100 cents = $1)

        // Calculate time until expiry
        let seconds_until_expiry = pair.expiration_time_secs.map(|exp| {
            if exp > now {
                exp - now
            } else {
                0
            }
        });

        // Capture is_live before moving pair
        let is_live = pair.is_live;

        // Calculate priority score
        let priority_score = Self::calculate_priority_score(
            is_live,
            seconds_until_expiry,
            profit_percent,
            config,
        );

        Self {
            request,
            pair,
            priority_score,
            profit_percent,
            seconds_until_expiry,
            is_live,
            queued_at_secs: now,
        }
    }

    /// Calculate priority score based on configured weights
    fn calculate_priority_score(
        is_live: bool,
        seconds_until_expiry: Option<u64>,
        profit_percent: f64,
        config: &PriorityConfig,
    ) -> f64 {
        let mut score = 0.0;

        // Live game boost (highest priority)
        if is_live {
            score += config.live_game_priority_boost * 100.0;
        }

        // Expiration urgency (inverse of time remaining)
        // Markets expiring sooner get higher scores
        if let Some(secs) = seconds_until_expiry {
            if secs > 0 {
                // Use logarithmic scale: closer expiration = higher score
                // Max score when secs = 60 (1 min), decreasing as secs increases
                let urgency = config.expiration_weight * (1000.0 / (secs as f64).max(1.0).sqrt());
                score += urgency;
            } else {
                // Already expired or about to expire - highest urgency
                score += config.expiration_weight * 1000.0;
            }
        }

        // Profit percentage (higher = better)
        score += config.profit_weight * profit_percent;

        score
    }

    /// Check if this opportunity meets the minimum profit threshold
    pub fn meets_profit_threshold(&self, config: &PriorityConfig) -> bool {
        self.profit_percent >= config.min_arb_percent
    }

    /// Check if this opportunity has sufficient liquidity on both sides
    pub fn has_sufficient_liquidity(&self, config: &PriorityConfig) -> bool {
        let min_size = self.request.yes_size.min(self.request.no_size) as u32 * 100; // Convert to cents
        config.meets_min_liquidity(min_size)
    }

    /// Get the clamped position size respecting max liquidity
    #[allow(dead_code)]
    pub fn clamped_position_size(&self, config: &PriorityConfig) -> (u16, bool) {
        let min_size_cents = self.request.yes_size.min(self.request.no_size) as u32 * 100;
        let (clamped, was_clamped) = config.clamp_liquidity(min_size_cents);
        ((clamped / 100) as u16, was_clamped)
    }

    /// Check if this game starts within the allowed time window (24 hours by default)
    /// Live games always pass. Games too far in the future are filtered out to avoid
    /// tying up capital.
    pub fn is_within_time_window(&self, config: &PriorityConfig) -> bool {
        config.is_within_time_window(self.pair.game_start_time_secs, self.is_live)
    }
}

// Implement ordering for BinaryHeap (max-heap by priority_score)
impl PartialEq for PrioritizedOpportunity {
    fn eq(&self, other: &Self) -> bool {
        self.priority_score == other.priority_score
    }
}

impl Eq for PrioritizedOpportunity {}

impl PartialOrd for PrioritizedOpportunity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedOpportunity {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher score = higher priority
        self.priority_score.partial_cmp(&other.priority_score)
            .unwrap_or(Ordering::Equal)
    }
}

/// Priority queue for arbitrage execution
#[allow(dead_code)]
pub struct PriorityQueue {
    /// The underlying heap
    heap: BinaryHeap<PrioritizedOpportunity>,
    /// Configuration
    config: PriorityConfig,
    /// Last sort timestamp
    last_sort_secs: u64,
}

impl PriorityQueue {
    /// Create a new priority queue
    pub fn new(config: PriorityConfig) -> Self {
        Self {
            heap: BinaryHeap::new(),
            config,
            last_sort_secs: current_unix_secs(),
        }
    }

    /// Push a new opportunity to the queue
    pub fn push(&mut self, opp: PrioritizedOpportunity) {
        // Filter out opportunities that don't meet thresholds
        if !opp.meets_profit_threshold(&self.config) {
            debug!("Rejecting opportunity: profit {}% < min {}%",
                   opp.profit_percent, self.config.min_arb_percent);
            return;
        }

        if !opp.has_sufficient_liquidity(&self.config) {
            debug!("Rejecting opportunity: insufficient liquidity");
            return;
        }

        // Filter out games starting too far in the future (default: >24 hours)
        // This prevents capital from being tied up in bets for distant games
        if !opp.is_within_time_window(&self.config) {
            debug!("Rejecting opportunity: game starts more than {}h away",
                   self.config.max_hours_until_game);
            return;
        }

        self.heap.push(opp);
    }

    /// Pop the highest priority opportunity
    pub fn pop(&mut self) -> Option<PrioritizedOpportunity> {
        self.heap.pop()
    }

    /// Peek at the highest priority opportunity without removing it
    pub fn peek(&self) -> Option<&PrioritizedOpportunity> {
        self.heap.peek()
    }

    /// Check if the queue needs re-sorting
    pub fn needs_resort(&self) -> bool {
        let now = current_unix_secs();
        now - self.last_sort_secs >= self.config.queue_sort_interval_secs
    }

    /// Resort the queue by recalculating all priority scores
    pub fn resort(&mut self) {
        if self.heap.is_empty() {
            self.last_sort_secs = current_unix_secs();
            return;
        }

        // Drain the heap, recalculate scores, and rebuild
        let items: Vec<_> = self.heap.drain().collect();

        for mut opp in items {
            // Recalculate priority score with current time
            let now = current_unix_secs();
            opp.seconds_until_expiry = opp.pair.expiration_time_secs.map(|exp| {
                if exp > now { exp - now } else { 0 }
            });
            opp.priority_score = PrioritizedOpportunity::calculate_priority_score(
                opp.is_live,
                opp.seconds_until_expiry,
                opp.profit_percent,
                &self.config,
            );

            // Re-add if still valid
            if opp.meets_profit_threshold(&self.config) {
                self.heap.push(opp);
            }
        }

        self.last_sort_secs = current_unix_secs();
    }

    /// Get the number of opportunities in the queue
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// Clear the queue
    pub fn clear(&mut self) {
        self.heap.clear();
    }
}

/// Shared priority queue wrapper for concurrent access
/// Uses parking_lot for faster synchronous locking and atomic resort check
pub struct SharedPriorityQueue {
    inner: RwLock<PriorityQueue>,
    config: PriorityConfig,
    /// Atomic timestamp for lockless resort check
    last_resort_secs: AtomicU64,
    /// Per-market cooldown tracking: market_id -> last_trade_timestamp
    market_cooldowns: RwLock<HashMap<u16, u64>>,
}

impl SharedPriorityQueue {
    /// Create a new shared priority queue
    pub fn new(config: PriorityConfig) -> Self {
        let queue = PriorityQueue::new(config.clone());
        Self {
            inner: RwLock::new(queue),
            config,
            last_resort_secs: AtomicU64::new(current_unix_secs()),
            market_cooldowns: RwLock::new(HashMap::new()),
        }
    }

    /// Check if a market is on cooldown
    #[inline]
    pub fn is_market_on_cooldown(&self, market_id: u16) -> bool {
        if self.config.market_cooldown_secs == 0 {
            return false; // Cooldowns disabled
        }

        let now = current_unix_secs();
        let cooldowns = self.market_cooldowns.read();
        if let Some(&last_trade) = cooldowns.get(&market_id) {
            now - last_trade < self.config.market_cooldown_secs
        } else {
            false
        }
    }

    /// Record a trade on a market (start cooldown)
    pub fn record_trade(&self, market_id: u16) {
        if self.config.market_cooldown_secs == 0 {
            return; // Cooldowns disabled
        }

        let now = current_unix_secs();
        self.market_cooldowns.write().insert(market_id, now);
    }

    /// Clean up expired cooldowns (call periodically)
    pub fn cleanup_cooldowns(&self) {
        if self.config.market_cooldown_secs == 0 {
            return;
        }

        let now = current_unix_secs();
        let mut cooldowns = self.market_cooldowns.write();
        cooldowns.retain(|_, &mut last_trade| {
            now - last_trade < self.config.market_cooldown_secs
        });
    }

    /// Push an opportunity to the queue (non-blocking with parking_lot)
    /// Checks cooldown before adding
    #[inline]
    pub async fn push(&self, opp: PrioritizedOpportunity) {
        // Check cooldown before adding
        if self.is_market_on_cooldown(opp.request.market_id) {
            debug!("Skipping market {} - on cooldown", opp.request.market_id);
            return;
        }
        // parking_lot locks are synchronous but fast - no need for async
        self.inner.write().push(opp);
    }

    /// Push an opportunity from an execution request
    /// Checks cooldown before adding
    #[inline]
    pub async fn push_from_request(&self, request: FastExecutionRequest, pair: Arc<MarketPair>) {
        // Check cooldown before creating opportunity
        if self.is_market_on_cooldown(request.market_id) {
            debug!("Skipping market {} - on cooldown", request.market_id);
            return;
        }
        let opp = PrioritizedOpportunity::new(request, pair, &self.config);
        self.push(opp).await;
    }

    /// Pop the highest priority opportunity
    /// Uses atomic check to avoid lock for resort decision
    #[inline]
    pub async fn pop(&self) -> Option<PrioritizedOpportunity> {
        // Fast atomic check for resort - avoids lock contention
        let now = current_unix_secs();
        let last = self.last_resort_secs.load(AtomicOrdering::Relaxed);

        if now - last >= self.config.queue_sort_interval_secs {
            // Try to claim the resort (compare-and-swap)
            if self.last_resort_secs.compare_exchange(
                last, now,
                AtomicOrdering::AcqRel,
                AtomicOrdering::Relaxed
            ).is_ok() {
                // We won the race - do the resort
                self.inner.write().resort();
            }
        }

        // Fast pop with parking_lot
        self.inner.write().pop()
    }

    /// Pop without resort check - for ultra-low-latency path
    #[inline]
    pub fn pop_fast(&self) -> Option<PrioritizedOpportunity> {
        self.inner.write().pop()
    }

    /// Get the number of opportunities in the queue
    pub async fn len(&self) -> usize {
        self.inner.read().len()
    }

    /// Check if the queue is empty
    pub async fn is_empty(&self) -> bool {
        self.inner.read().is_empty()
    }

    /// Get configuration reference
    #[inline]
    pub fn config(&self) -> &PriorityConfig {
        &self.config
    }

    /// Force a resort of the queue (for background task)
    #[inline]
    pub fn force_resort(&self) -> (usize, usize) {
        let mut inner = self.inner.write();
        let before = inner.len();
        if !inner.is_empty() {
            inner.resort();
        }
        let after = inner.len();
        self.last_resort_secs.store(current_unix_secs(), AtomicOrdering::Release);
        (before, after)
    }
}

/// Scan all markets for arbitrage opportunities and queue them by priority
pub async fn scan_and_queue_opportunities(
    state: &GlobalState,
    queue: &SharedPriorityQueue,
    threshold_cents: u16,
) -> usize {
    let now = current_unix_secs();
    let config = queue.config();
    let mut queued = 0;

    for market_id in 0..state.market_count() {
        if let Some(market) = state.get_by_id(market_id as u16) {
            let arb_mask = market.check_arbs(threshold_cents);
            if arb_mask == 0 {
                continue;
            }

            let pair = match &market.pair {
                Some(p) => p.clone(),
                None => continue,
            };

            let (k_yes, k_no, k_yes_sz, k_no_sz) = market.kalshi.load();
            let (p_yes, p_no, p_yes_sz, p_no_sz) = market.poly.load();

            // Check each arb type and queue if profitable
            use crate::types::ArbType;

            // Bit 0: Poly YES + Kalshi NO
            if arb_mask & 1 != 0 {
                let k_no_fee = kalshi_fee_cents(k_no);
                let total_cost = p_yes + k_no + k_no_fee;
                let profit = 100 - total_cost as i16;

                if profit >= config.min_arb_percent as i16 {
                    let request = FastExecutionRequest {
                        market_id: market_id as u16,
                        yes_price: p_yes,
                        no_price: k_no,
                        yes_size: p_yes_sz,
                        no_size: k_no_sz,
                        arb_type: ArbType::PolyYesKalshiNo,
                        detected_ns: now,
                    };
                    queue.push_from_request(request, pair.clone()).await;
                    queued += 1;
                }
            }

            // Bit 1: Kalshi YES + Poly NO
            if arb_mask & 2 != 0 {
                let k_yes_fee = kalshi_fee_cents(k_yes);
                let total_cost = k_yes + k_yes_fee + p_no;
                let profit = 100 - total_cost as i16;

                if profit >= config.min_arb_percent as i16 {
                    let request = FastExecutionRequest {
                        market_id: market_id as u16,
                        yes_price: k_yes,
                        no_price: p_no,
                        yes_size: k_yes_sz,
                        no_size: p_no_sz,
                        arb_type: ArbType::KalshiYesPolyNo,
                        detected_ns: now,
                    };
                    queue.push_from_request(request, pair.clone()).await;
                    queued += 1;
                }
            }

            // Bit 2: Poly-only
            if arb_mask & 4 != 0 {
                let total_cost = p_yes + p_no;
                let profit = 100 - total_cost as i16;

                if profit >= config.min_arb_percent as i16 {
                    let request = FastExecutionRequest {
                        market_id: market_id as u16,
                        yes_price: p_yes,
                        no_price: p_no,
                        yes_size: p_yes_sz,
                        no_size: p_no_sz,
                        arb_type: ArbType::PolyOnly,
                        detected_ns: now,
                    };
                    queue.push_from_request(request, pair.clone()).await;
                    queued += 1;
                }
            }

            // Bit 3: Kalshi-only
            if arb_mask & 8 != 0 {
                let k_yes_fee = kalshi_fee_cents(k_yes);
                let k_no_fee = kalshi_fee_cents(k_no);
                let total_cost = k_yes + k_yes_fee + k_no + k_no_fee;
                let profit = 100 - total_cost as i16;

                if profit >= config.min_arb_percent as i16 {
                    let request = FastExecutionRequest {
                        market_id: market_id as u16,
                        yes_price: k_yes,
                        no_price: k_no,
                        yes_size: k_yes_sz,
                        no_size: k_no_sz,
                        arb_type: ArbType::KalshiOnly,
                        detected_ns: now,
                    };
                    queue.push_from_request(request, pair.clone()).await;
                    queued += 1;
                }
            }
        }
    }

    if queued > 0 {
        info!("[PRIORITY] Queued {} opportunities", queued);
    }

    queued
}

/// Background task that periodically re-sorts the queue
pub async fn priority_resort_loop(queue: Arc<SharedPriorityQueue>) {
    let interval_secs = queue.config().queue_sort_interval_secs;
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval_secs));

    info!("[PRIORITY] Resort loop started (interval: {}s)", interval_secs);

    loop {
        interval.tick().await;

        // Use force_resort method (parking_lot locks are synchronous but fast)
        let (before_len, after_len) = queue.force_resort();

        if before_len != after_len {
            debug!("[PRIORITY] Resorted queue: {} -> {} opportunities", before_len, after_len);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{MarketType, ArbType};

    fn make_test_pair(is_live: bool, expiry_secs: Option<u64>) -> Arc<MarketPair> {
        // For tests, game_start_time is same as expiry (simplified)
        Arc::new(MarketPair {
            pair_id: "test".into(),
            league: "epl".into(),
            market_type: MarketType::Moneyline,
            description: "Test Match".into(),
            kalshi_event_ticker: "TEST".into(),
            kalshi_market_ticker: "TEST-YES".into(),
            poly_slug: "test".into(),
            poly_yes_token: "yes".into(),
            poly_no_token: "no".into(),
            line_value: None,
            team_suffix: None,
            game_start_time_secs: expiry_secs,
            expiration_time_secs: expiry_secs,
            is_live,
        })
    }

    fn make_test_request(profit_cents: i16) -> FastExecutionRequest {
        // profit = 100 - yes - no - fee
        // So for 8 cents profit: 100 - 40 - 50 - 2 = 8
        let yes_price = 40;
        let no_price = (100 - profit_cents - yes_price as i16 - 2) as u16; // -2 for estimated fee

        FastExecutionRequest {
            market_id: 0,
            yes_price: yes_price,
            no_price: no_price,
            yes_size: 1000,
            no_size: 1000,
            arb_type: ArbType::PolyYesKalshiNo,
            detected_ns: 0,
        }
    }

    #[test]
    fn test_priority_scoring_live_games() {
        let config = PriorityConfig::default();
        let now = current_unix_secs();

        // Live game should have higher priority
        let live_pair = make_test_pair(true, Some(now + 3600));
        let non_live_pair = make_test_pair(false, Some(now + 3600));

        let request = make_test_request(8);

        let live_opp = PrioritizedOpportunity::new(request.clone(), live_pair, &config);
        let non_live_opp = PrioritizedOpportunity::new(request, non_live_pair, &config);

        assert!(live_opp.priority_score > non_live_opp.priority_score,
                "Live game should have higher priority");
    }

    #[test]
    fn test_priority_scoring_expiration() {
        let config = PriorityConfig::default();
        let now = current_unix_secs();

        // Earlier expiration should have higher priority
        let soon_pair = make_test_pair(false, Some(now + 300));    // 5 minutes
        let later_pair = make_test_pair(false, Some(now + 3600));  // 1 hour

        let request = make_test_request(8);

        let soon_opp = PrioritizedOpportunity::new(request.clone(), soon_pair, &config);
        let later_opp = PrioritizedOpportunity::new(request, later_pair, &config);

        assert!(soon_opp.priority_score > later_opp.priority_score,
                "Sooner expiration should have higher priority");
    }

    #[test]
    fn test_priority_scoring_profit() {
        let config = PriorityConfig::default();
        let now = current_unix_secs();

        // Higher profit should have higher priority (with same expiration)
        let pair = make_test_pair(false, Some(now + 3600));

        let high_profit_req = make_test_request(10);
        let low_profit_req = make_test_request(5);

        let high_opp = PrioritizedOpportunity::new(high_profit_req, pair.clone(), &config);
        let low_opp = PrioritizedOpportunity::new(low_profit_req, pair, &config);

        assert!(high_opp.priority_score > low_opp.priority_score,
                "Higher profit should have higher priority");
    }

    #[test]
    fn test_min_profit_threshold() {
        let config = PriorityConfig {
            min_arb_percent: 2.0, // Require 2% minimum
            ..Default::default()
        };

        let pair = make_test_pair(false, None);

        let good_req = make_test_request(3);  // 3% profit
        let bad_req = make_test_request(1);   // 1% profit

        let good_opp = PrioritizedOpportunity::new(good_req, pair.clone(), &config);
        let bad_opp = PrioritizedOpportunity::new(bad_req, pair, &config);

        assert!(good_opp.meets_profit_threshold(&config));
        assert!(!bad_opp.meets_profit_threshold(&config));
    }

    #[test]
    fn test_priority_queue_ordering() {
        let config = PriorityConfig::default();
        let now = current_unix_secs();
        let mut queue = PriorityQueue::new(config.clone());

        // Add opportunities with different priorities
        let live_pair = make_test_pair(true, Some(now + 3600));
        let soon_pair = make_test_pair(false, Some(now + 300));
        let later_pair = make_test_pair(false, Some(now + 7200));

        let request = make_test_request(8);

        queue.push(PrioritizedOpportunity::new(request.clone(), later_pair, &config));
        queue.push(PrioritizedOpportunity::new(request.clone(), live_pair, &config));
        queue.push(PrioritizedOpportunity::new(request, soon_pair, &config));

        // Live game should pop first
        let first = queue.pop().unwrap();
        assert!(first.is_live, "Live game should be first");

        // Then sooner expiration
        let second = queue.pop().unwrap();
        assert!(second.seconds_until_expiry.unwrap() < 600, "Sooner expiration should be second");
    }
}
