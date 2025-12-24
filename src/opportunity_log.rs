//! Opportunity scanner and near-miss logger for threshold optimization.
//!
//! This module logs ALL arbitrage opportunities scanned (including rejected ones)
//! to enable backtesting and optimal threshold discovery.

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::{self, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::ml_optimizer::{MarketCategory, OpportunityData};

/// Maximum opportunities to keep in memory (50K = ~2 weeks of data at high volume)
const MAX_OPPORTUNITIES: usize = 50000;

/// A scanned opportunity (may or may not have been executed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedOpportunity {
    pub timestamp: String,
    pub market_id: u16,
    pub market_name: String,
    /// Parsed category for ML optimization
    pub sport: String,
    pub bet_type: String,
    /// Raw prices before any adjustments
    pub kalshi_yes: u16,
    pub kalshi_no: u16,
    pub poly_yes: u16,
    pub poly_no: u16,
    /// Best combination found
    pub best_combo: String, // "PolyYes+KalshiNo", "KalshiYes+PolyNo", "PolyOnly", "KalshiOnly"
    pub raw_total_cents: u16, // yes + no (no fees)
    pub kalshi_fee_cents: u16,
    pub adjusted_total_cents: u16, // with fees
    /// Liquidity available
    pub yes_liquidity_cents: u16,
    pub no_liquidity_cents: u16,
    pub min_liquidity_cents: u16,
    /// What happened
    pub was_executed: bool,
    pub rejection_reason: Option<String>,
    /// Simulated profits at different thresholds (pre-calculated)
    pub profit_at_100: i16, // If threshold was 100¢ (any profit)
    pub profit_at_99: i16,  // If threshold was 99¢ (1% profit)
    pub profit_at_98: i16,  // If threshold was 98¢ (2% profit)
    pub profit_at_995: i16, // Current default (0.5% profit)
}

/// Threshold simulation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdSimulation {
    pub threshold_cents: u16,
    pub threshold_name: String,
    pub opportunities_found: u32,
    pub total_profit_cents: i64,
    pub avg_profit_cents: f64,
    pub total_volume_cents: u64,
    pub min_liquidity_used: u32, // The min liquidity setting
}

/// Analysis of optimal settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimalSettings {
    pub generated_at: String,
    pub analysis_period_hours: f64,
    pub total_opportunities_scanned: u32,
    /// Simulations at different threshold levels
    pub simulations: Vec<ThresholdSimulation>,
    /// Recommended settings
    pub recommended_threshold_cents: u16,
    pub recommended_min_liquidity_cents: u32,
    pub reasoning: String,
    /// Comparison with current
    pub current_profit_cents: i64,
    pub optimal_profit_cents: i64,
    pub improvement_percent: f64,
}

/// Opportunity logger
pub struct OpportunityLogger {
    file_path: String,
    opportunities: Mutex<VecDeque<ScannedOpportunity>>,
    start_time: chrono::DateTime<chrono::Utc>,
}

impl OpportunityLogger {
    pub fn new(base_dir: &str) -> Self {
        let file_path = format!("{}/opportunities.json", base_dir);

        // Create directory if needed
        if let Some(parent) = Path::new(&file_path).parent() {
            let _ = fs::create_dir_all(parent);
        }

        // Load existing or start fresh
        let opportunities = if Path::new(&file_path).exists() {
            match fs::read_to_string(&file_path) {
                Ok(content) => {
                    serde_json::from_str::<VecDeque<ScannedOpportunity>>(&content)
                        .unwrap_or_default()
                }
                Err(_) => VecDeque::new(),
            }
        } else {
            VecDeque::new()
        };

        Self {
            file_path,
            opportunities: Mutex::new(opportunities),
            start_time: chrono::Utc::now(),
        }
    }

    /// Log a scanned opportunity
    pub fn log_opportunity(&self, opp: ScannedOpportunity) {
        let mut opps = self.opportunities.lock();

        // Keep only recent opportunities
        while opps.len() >= MAX_OPPORTUNITIES {
            opps.pop_front();
        }

        opps.push_back(opp);

        // Save periodically (every 100 entries)
        if opps.len() % 100 == 0 {
            self.save_locked(&opps);
        }
    }

    fn save_locked(&self, opps: &VecDeque<ScannedOpportunity>) {
        if let Ok(file) = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.file_path)
        {
            let mut writer = BufWriter::new(file);
            let _ = serde_json::to_writer(&mut writer, opps);
            let _ = writer.flush();
        }
    }

    /// Save to disk
    pub fn save(&self) {
        let opps = self.opportunities.lock();
        self.save_locked(&opps);
    }

    /// Run threshold simulation on logged opportunities
    pub fn run_simulation(&self, min_liquidity_cents: u32) -> OptimalSettings {
        let opps = self.opportunities.lock();
        let now = chrono::Utc::now();
        let hours = (now - self.start_time).num_seconds() as f64 / 3600.0;

        // Define thresholds to test
        let thresholds = vec![
            (100, "Any Profit (100¢)"),
            (99, "1% Profit (99¢)"),
            (98, "2% Profit (98¢)"),
            (995, "0.5% Profit (99.5¢)"), // Note: We use 995 to represent 99.5
            (97, "3% Profit (97¢)"),
        ];

        let mut simulations: Vec<ThresholdSimulation> = Vec::new();

        for (threshold, name) in &thresholds {
            let _threshold_cents = if *threshold == 995 { 99 } else { *threshold }; // Normalize

            let mut count = 0u32;
            let mut total_profit = 0i64;
            let mut total_volume = 0u64;

            for opp in opps.iter() {
                // Skip if below min liquidity
                if opp.min_liquidity_cents < min_liquidity_cents as u16 {
                    continue;
                }

                // Check if this would have been profitable at this threshold
                let profit = match *threshold {
                    100 => opp.profit_at_100,
                    99 => opp.profit_at_99,
                    98 => opp.profit_at_98,
                    995 => opp.profit_at_995,
                    97 => (100 - opp.adjusted_total_cents as i16).max(-50),
                    _ => 0,
                };

                let would_execute = match *threshold {
                    100 => opp.adjusted_total_cents < 100,
                    99 => opp.adjusted_total_cents < 99,
                    98 => opp.adjusted_total_cents < 98,
                    995 => opp.adjusted_total_cents < 100, // 99.5 rounds to <100
                    97 => opp.adjusted_total_cents < 97,
                    _ => false,
                };

                if would_execute && profit > 0 {
                    count += 1;
                    total_profit += profit as i64;
                    total_volume += opp.min_liquidity_cents as u64;
                }
            }

            simulations.push(ThresholdSimulation {
                threshold_cents: *threshold as u16,
                threshold_name: name.to_string(),
                opportunities_found: count,
                total_profit_cents: total_profit,
                avg_profit_cents: if count > 0 { total_profit as f64 / count as f64 } else { 0.0 },
                total_volume_cents: total_volume,
                min_liquidity_used: min_liquidity_cents,
            });
        }

        // Find optimal (highest total profit)
        let best = simulations.iter()
            .max_by_key(|s| s.total_profit_cents)
            .cloned();

        let current = simulations.iter()
            .find(|s| s.threshold_cents == 995 || s.threshold_cents == 99)
            .cloned();

        let (recommended_threshold, reasoning, optimal_profit, current_profit) = if let Some(b) = best {
            let curr_profit = current.map(|c| c.total_profit_cents).unwrap_or(0);
            let _improvement = if curr_profit > 0 {
                ((b.total_profit_cents - curr_profit) as f64 / curr_profit as f64) * 100.0
            } else {
                0.0
            };

            let reason = if b.threshold_cents < 99 {
                format!(
                    "Lower threshold ({}¢) would find {} more opportunities for {}¢ more profit",
                    b.threshold_cents, b.opportunities_found, b.total_profit_cents - curr_profit
                )
            } else if b.threshold_cents > 99 {
                format!(
                    "Higher threshold ({}¢) reduces risk while maintaining {}¢ profit",
                    b.threshold_cents, b.total_profit_cents
                )
            } else {
                "Current settings appear optimal based on available data".to_string()
            };

            (b.threshold_cents, reason, b.total_profit_cents, curr_profit)
        } else {
            (99, "Insufficient data for recommendation".to_string(), 0, 0)
        };

        OptimalSettings {
            generated_at: now.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            analysis_period_hours: hours,
            total_opportunities_scanned: opps.len() as u32,
            simulations,
            recommended_threshold_cents: recommended_threshold,
            recommended_min_liquidity_cents: min_liquidity_cents,
            reasoning,
            current_profit_cents: current_profit,
            optimal_profit_cents: optimal_profit,
            improvement_percent: if current_profit > 0 {
                ((optimal_profit - current_profit) as f64 / current_profit as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Get count of logged opportunities
    pub fn count(&self) -> usize {
        self.opportunities.lock().len()
    }

    /// Get opportunities as OpportunityData for ML optimizer
    pub fn get_opportunity_data(&self) -> Vec<OpportunityData> {
        let opps = self.opportunities.lock();
        opps.iter().map(|opp| OpportunityData {
            timestamp: opp.timestamp.clone(),
            market_name: opp.market_name.clone(),
            adjusted_cost_cents: opp.adjusted_total_cents,
            liquidity_cents: opp.min_liquidity_cents as u32,
            was_executed: opp.was_executed,
        }).collect()
    }

    /// Get opportunities grouped by sport
    pub fn get_by_sport(&self) -> std::collections::HashMap<String, Vec<ScannedOpportunity>> {
        let opps = self.opportunities.lock();
        let mut by_sport: std::collections::HashMap<String, Vec<ScannedOpportunity>> = std::collections::HashMap::new();

        for opp in opps.iter() {
            by_sport.entry(opp.sport.clone()).or_default().push(opp.clone());
        }

        by_sport
    }
}

// Global opportunity logger
static OPPORTUNITY_LOGGER: std::sync::OnceLock<OpportunityLogger> = std::sync::OnceLock::new();

/// Initialize the global opportunity logger
pub fn init_opportunity_logger(base_dir: &str) -> &'static OpportunityLogger {
    OPPORTUNITY_LOGGER.get_or_init(|| OpportunityLogger::new(base_dir))
}

/// Get the global opportunity logger
pub fn get_opportunity_logger() -> Option<&'static OpportunityLogger> {
    OPPORTUNITY_LOGGER.get()
}

/// Helper to calculate Kalshi fee for a price
#[inline]
pub fn kalshi_fee(price_cents: u16) -> u16 {
    if price_cents == 0 || price_cents >= 100 {
        return 0;
    }
    // fee = ceil(7 × p × (100-p) / 10000)
    let p = price_cents as u32;
    let numerator = 7 * p * (100 - p) + 9999;
    (numerator / 10000) as u16
}

/// Create a ScannedOpportunity from market prices
pub fn create_opportunity(
    market_id: u16,
    market_name: &str,
    kalshi_yes: u16,
    kalshi_no: u16,
    poly_yes: u16,
    poly_no: u16,
    yes_size: u16,
    no_size: u16,
    was_executed: bool,
    rejection_reason: Option<&str>,
) -> ScannedOpportunity {
    // Parse market category
    let category = MarketCategory::from_market_name(market_name);

    // Calculate best combination
    let k_yes_fee = kalshi_fee(kalshi_yes);
    let k_no_fee = kalshi_fee(kalshi_no);

    // Cross-platform options
    let poly_yes_kalshi_no = poly_yes + kalshi_no + k_no_fee;
    let kalshi_yes_poly_no = kalshi_yes + k_yes_fee + poly_no;

    // Same-platform options
    let poly_only = poly_yes + poly_no; // No fees on Polymarket
    let kalshi_only = kalshi_yes + kalshi_no + k_yes_fee + k_no_fee;

    // Find best
    let (best_combo, raw_total, fee, adjusted_total) =
        if poly_yes_kalshi_no <= kalshi_yes_poly_no &&
           poly_yes_kalshi_no <= poly_only &&
           poly_yes_kalshi_no <= kalshi_only {
            ("PolyYes+KalshiNo", poly_yes + kalshi_no, k_no_fee, poly_yes_kalshi_no)
        } else if kalshi_yes_poly_no <= poly_only && kalshi_yes_poly_no <= kalshi_only {
            ("KalshiYes+PolyNo", kalshi_yes + poly_no, k_yes_fee, kalshi_yes_poly_no)
        } else if poly_only <= kalshi_only {
            ("PolyOnly", poly_only, 0, poly_only)
        } else {
            ("KalshiOnly", kalshi_yes + kalshi_no, k_yes_fee + k_no_fee, kalshi_only)
        };

    let min_liq = yes_size.min(no_size);

    ScannedOpportunity {
        timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        market_id,
        market_name: market_name.to_string(),
        sport: category.sport,
        bet_type: category.bet_type,
        kalshi_yes,
        kalshi_no,
        poly_yes,
        poly_no,
        best_combo: best_combo.to_string(),
        raw_total_cents: raw_total,
        kalshi_fee_cents: fee,
        adjusted_total_cents: adjusted_total,
        yes_liquidity_cents: yes_size,
        no_liquidity_cents: no_size,
        min_liquidity_cents: min_liq,
        was_executed,
        rejection_reason: rejection_reason.map(String::from),
        profit_at_100: (100 - adjusted_total as i16).max(-50),
        profit_at_99: (99 - adjusted_total as i16).max(-50),
        profit_at_98: (98 - adjusted_total as i16).max(-50),
        profit_at_995: (100 - adjusted_total as i16).max(-50), // Same as 100 for comparison
    }
}
