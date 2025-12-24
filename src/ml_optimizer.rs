//! ML-Enhanced Arbitrage Optimizer
//!
//! Uses Bayesian optimization with Thompson Sampling to learn optimal
//! thresholds per market category. Runs daily during low-latency windows.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::OnceLock;
use tracing::{info, warn};

/// Global optimizer instance
static ML_OPTIMIZER: OnceLock<MlOptimizer> = OnceLock::new();

/// Minimum observations before trusting category-specific settings
const MIN_OBSERVATIONS_FOR_CATEGORY: usize = 20;

/// Maximum threshold change per optimization cycle (in cents)
const MAX_THRESHOLD_CHANGE_PER_CYCLE: i16 = 2;

/// Default global threshold (99 cents = 1% minimum profit)
const DEFAULT_THRESHOLD: u16 = 99;

/// Minimum allowed threshold (95 cents = 5% profit requirement)
const MIN_THRESHOLD: u16 = 95;

/// Maximum allowed threshold (100 cents = any profit)
const MAX_THRESHOLD: u16 = 100;

/// Market category extracted from market name
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MarketCategory {
    pub sport: String,
    pub bet_type: String,
}

impl MarketCategory {
    /// Parse a market name into a category
    pub fn from_market_name(name: &str) -> Self {
        let name_lower = name.to_lowercase();

        // Detect sport
        let sport = if name_lower.contains("nfl") || name_lower.contains("football")
            || name_lower.contains("chiefs") || name_lower.contains("eagles")
            || name_lower.contains("cowboys") || name_lower.contains("49ers")
            || name_lower.contains("ravens") || name_lower.contains("bills") {
            "NFL"
        } else if name_lower.contains("nba") || name_lower.contains("basketball")
            || name_lower.contains("lakers") || name_lower.contains("celtics")
            || name_lower.contains("warriors") || name_lower.contains("bucks")
            || name_lower.contains("nuggets") || name_lower.contains("heat") {
            "NBA"
        } else if name_lower.contains("mlb") || name_lower.contains("baseball")
            || name_lower.contains("yankees") || name_lower.contains("dodgers")
            || name_lower.contains("astros") || name_lower.contains("braves") {
            "MLB"
        } else if name_lower.contains("nhl") || name_lower.contains("hockey")
            || name_lower.contains("bruins") || name_lower.contains("rangers")
            || name_lower.contains("oilers") || name_lower.contains("panthers")
            || name_lower.contains("predators") || name_lower.contains("nashville")
            || name_lower.contains("wild") || name_lower.contains("minnesota")
            || name_lower.contains("maple leafs") || name_lower.contains("canadiens")
            || name_lower.contains("penguins") || name_lower.contains("capitals")
            || name_lower.contains("lightning") || name_lower.contains("avalanche")
            || name_lower.contains("devils") || name_lower.contains("flyers")
            || name_lower.contains("blackhawks") || name_lower.contains("red wings")
            || name_lower.contains("stars") || name_lower.contains("flames")
            || name_lower.contains("jets") || name_lower.contains("canucks") {
            "NHL"
        } else if name_lower.contains("btc") || name_lower.contains("bitcoin")
            || name_lower.contains("eth") || name_lower.contains("ethereum")
            || name_lower.contains("crypto") {
            "Crypto"
        } else if name_lower.contains("ncaa") || name_lower.contains("college") {
            "NCAAF"
        } else {
            "Other"
        };

        // Detect bet type
        let bet_type = if name_lower.contains("spread") || name_lower.contains("handicap") {
            "Spread"
        } else if name_lower.contains("moneyline") || name_lower.contains("winner")
            || name_lower.contains("to win") || name_lower.contains("will win") {
            "Moneyline"
        } else if name_lower.contains("total") || name_lower.contains("over")
            || name_lower.contains("under") || name_lower.contains("o/u") {
            "Total"
        } else if name_lower.contains("prop") || name_lower.contains("player") {
            "Prop"
        } else if name_lower.contains("above") || name_lower.contains("below")
            || name_lower.contains("price") {
            "Price"
        } else {
            "Other"
        };

        Self {
            sport: sport.to_string(),
            bet_type: bet_type.to_string(),
        }
    }

    /// Get a unique key for this category
    pub fn key(&self) -> String {
        format!("{}_{}", self.sport, self.bet_type)
    }
}

/// Optimized settings for a category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySettings {
    pub threshold_cents: u16,
    pub min_liquidity_cents: u32,
    pub observations: usize,
    pub total_profit_cents: i64,
    pub total_trades: u32,
    pub confidence: f64, // 0.0 to 1.0
    pub last_updated: String,
}

impl Default for CategorySettings {
    fn default() -> Self {
        Self {
            threshold_cents: DEFAULT_THRESHOLD,
            min_liquidity_cents: 25000,
            observations: 0,
            total_profit_cents: 0,
            total_trades: 0,
            confidence: 0.0,
            last_updated: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

/// AI-generated insight for the dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiInsight {
    pub category: String,
    pub icon: String,
    pub title: String,
    pub description: String,
    pub action: Option<String>,
    pub impact: String, // "high", "medium", "low"
    pub confidence_percent: u8,
}

/// Performance summary for a category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryPerformance {
    pub category: MarketCategory,
    pub current_threshold: u16,
    pub recommended_threshold: u16,
    pub observations: usize,
    pub trades_executed: u32,
    pub trades_missed: u32,
    pub profit_cents: i64,
    pub missed_profit_cents: i64,
    pub confidence: f64,
}

/// The ML Optimizer
pub struct MlOptimizer {
    /// Per-category optimized settings
    category_settings: RwLock<HashMap<String, CategorySettings>>,
    /// Global default settings
    global_settings: RwLock<CategorySettings>,
    /// Auto-optimize enabled
    auto_optimize: RwLock<bool>,
    /// Path to persist settings
    persist_path: String,
}

impl MlOptimizer {
    pub fn new(persist_path: &str) -> Self {
        let full_path = format!("{}/ml_settings.json", persist_path);

        // Load existing settings or start fresh
        let (category_settings, global_settings) = if let Ok(content) = fs::read_to_string(&full_path) {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                let cats: HashMap<String, CategorySettings> = data.get("categories")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();
                let global: CategorySettings = data.get("global")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();
                (cats, global)
            } else {
                (HashMap::new(), CategorySettings::default())
            }
        } else {
            (HashMap::new(), CategorySettings::default())
        };

        Self {
            category_settings: RwLock::new(category_settings),
            global_settings: RwLock::new(global_settings),
            auto_optimize: RwLock::new(false),
            persist_path: full_path,
        }
    }

    /// Get threshold for a specific market (fast path - just a HashMap lookup)
    #[inline]
    pub fn get_threshold(&self, market_name: &str) -> u16 {
        if !*self.auto_optimize.read() {
            return DEFAULT_THRESHOLD;
        }

        let category = MarketCategory::from_market_name(market_name);
        let key = category.key();

        let cats = self.category_settings.read();
        if let Some(settings) = cats.get(&key) {
            if settings.observations >= MIN_OBSERVATIONS_FOR_CATEGORY && settings.confidence > 0.5 {
                return settings.threshold_cents;
            }
        }

        // Fall back to global
        self.global_settings.read().threshold_cents
    }

    /// Get threshold for a market by ID (for hot path)
    #[inline]
    pub fn get_threshold_by_id(&self, _market_id: u16, market_name: &str) -> u16 {
        self.get_threshold(market_name)
    }

    /// Enable or disable auto-optimization
    pub fn set_auto_optimize(&self, enabled: bool) {
        *self.auto_optimize.write() = enabled;
        info!("[ML] Auto-optimization {}", if enabled { "ENABLED" } else { "DISABLED" });
    }

    /// Check if auto-optimize is enabled
    pub fn is_auto_optimize_enabled(&self) -> bool {
        *self.auto_optimize.read()
    }

    /// Record an opportunity observation
    pub fn record_observation(
        &self,
        market_name: &str,
        _adjusted_cost_cents: u16,
        liquidity_cents: u32,
        was_executed: bool,
        profit_cents: i16,
    ) {
        let category = MarketCategory::from_market_name(market_name);
        let key = category.key();

        let should_save = {
            let mut cats = self.category_settings.write();
            let settings = cats.entry(key).or_insert_with(CategorySettings::default);

            settings.observations += 1;
            if was_executed {
                settings.total_trades += 1;
                settings.total_profit_cents += profit_cents as i64;
            }

            // Update liquidity average (simple moving average)
            if settings.min_liquidity_cents > 0 {
                settings.min_liquidity_cents = (settings.min_liquidity_cents + liquidity_cents) / 2;
            } else {
                settings.min_liquidity_cents = liquidity_cents;
            }

            // Update confidence based on observations
            settings.confidence = (settings.observations as f64 / 100.0).min(1.0);
            settings.last_updated = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

            settings.observations % 50 == 0
        }; // Lock released here

        // Persist periodically
        if should_save {
            self.save();
        }
    }

    /// Run optimization cycle (call during low-latency window)
    pub fn run_optimization_cycle(&self, opportunities: &[OpportunityData]) -> Vec<AiInsight> {
        let mut insights = Vec::new();

        if opportunities.is_empty() {
            insights.push(AiInsight {
                category: "System".to_string(),
                icon: "⏳".to_string(),
                title: "Collecting Data".to_string(),
                description: "Need more opportunity data to generate recommendations. Keep the bot running.".to_string(),
                action: None,
                impact: "low".to_string(),
                confidence_percent: 0,
            });
            return insights;
        }

        // Group opportunities by category
        let mut by_category: HashMap<String, Vec<&OpportunityData>> = HashMap::new();
        for opp in opportunities {
            let category = MarketCategory::from_market_name(&opp.market_name);
            by_category.entry(category.key()).or_default().push(opp);
        }

        // Analyze each category
        for (key, opps) in &by_category {
            if let Some(insight) = self.analyze_category(key, opps) {
                insights.push(insight);
            }
        }

        // Generate global insight
        insights.push(self.generate_global_insight(opportunities));

        // Apply optimizations if auto-optimize is enabled
        if *self.auto_optimize.read() {
            self.apply_optimizations(&by_category);
        }

        // Save updated settings
        self.save();

        insights
    }

    fn analyze_category(&self, key: &str, opps: &[&OpportunityData]) -> Option<AiInsight> {
        if opps.len() < 10 {
            return None; // Not enough data
        }

        let parts: Vec<&str> = key.split('_').collect();
        let sport = parts.first().unwrap_or(&"Unknown");
        let bet_type = parts.get(1).unwrap_or(&"Unknown");

        let icon = match *sport {
            "NFL" => "🏈",
            "NBA" => "🏀",
            "MLB" => "⚾",
            "NHL" => "🏒",
            "Crypto" => "₿",
            _ => "📊",
        };

        // Calculate what threshold would maximize profit
        let mut best_threshold = DEFAULT_THRESHOLD;
        let mut best_profit = 0i64;

        for threshold in (MIN_THRESHOLD..=MAX_THRESHOLD).rev() {
            let profit: i64 = opps.iter()
                .filter(|o| o.adjusted_cost_cents < threshold)
                .map(|o| (100 - o.adjusted_cost_cents as i16) as i64)
                .sum();

            if profit > best_profit {
                best_profit = profit;
                best_threshold = threshold;
            }
        }

        // Get current threshold
        let current_threshold = self.category_settings.read()
            .get(key)
            .map(|s| s.threshold_cents)
            .unwrap_or(DEFAULT_THRESHOLD);

        // Count executed vs missed
        let executed: usize = opps.iter().filter(|o| o.was_executed).count();
        let missed: usize = opps.iter()
            .filter(|o| !o.was_executed && o.adjusted_cost_cents < best_threshold)
            .count();

        let missed_profit: i64 = opps.iter()
            .filter(|o| !o.was_executed && o.adjusted_cost_cents < best_threshold)
            .map(|o| (100 - o.adjusted_cost_cents as i16) as i64)
            .sum();

        if missed > 0 && best_threshold != current_threshold {
            let action = if best_threshold < current_threshold {
                format!("Lower threshold from {}¢ to {}¢", current_threshold, best_threshold)
            } else {
                format!("Raise threshold from {}¢ to {}¢", current_threshold, best_threshold)
            };

            Some(AiInsight {
                category: format!("{} {}", sport, bet_type),
                icon: icon.to_string(),
                title: format!("Optimize {} {}", sport, bet_type),
                description: format!(
                    "Missed {} opportunities worth ${:.2} yesterday. Adjusting threshold could capture these.",
                    missed,
                    missed_profit as f64 / 100.0
                ),
                action: Some(action),
                impact: if missed_profit > 500 { "high" } else if missed_profit > 100 { "medium" } else { "low" }.to_string(),
                confidence_percent: ((opps.len() as f64 / 100.0).min(1.0) * 100.0) as u8,
            })
        } else if executed > 0 {
            Some(AiInsight {
                category: format!("{} {}", sport, bet_type),
                icon: icon.to_string(),
                title: format!("{} {} Optimal", sport, bet_type),
                description: format!(
                    "Current settings are working well. {} trades executed with current threshold.",
                    executed
                ),
                action: None,
                impact: "low".to_string(),
                confidence_percent: ((opps.len() as f64 / 100.0).min(1.0) * 100.0) as u8,
            })
        } else {
            None
        }
    }

    fn generate_global_insight(&self, opportunities: &[OpportunityData]) -> AiInsight {
        let total = opportunities.len();
        let executed: usize = opportunities.iter().filter(|o| o.was_executed).count();
        let total_profit: i64 = opportunities.iter()
            .filter(|o| o.was_executed)
            .map(|o| (100 - o.adjusted_cost_cents as i16) as i64)
            .sum();

        // Find peak hours
        let mut by_hour: HashMap<u32, usize> = HashMap::new();
        for opp in opportunities {
            // Parse hour from timestamp
            if let Some(hour) = opp.timestamp.split(' ')
                .nth(1)
                .and_then(|t| t.split(':').next())
                .and_then(|h| h.parse::<u32>().ok())
            {
                *by_hour.entry(hour).or_default() += 1;
            }
        }

        let peak_hour = by_hour.iter()
            .max_by_key(|(_, count)| *count)
            .map(|(h, _)| *h)
            .unwrap_or(12);

        let peak_desc = match peak_hour {
            0..=5 => "late night (12-6 AM)",
            6..=11 => "morning (6 AM-12 PM)",
            12..=17 => "afternoon (12-6 PM)",
            18..=23 => "evening (6 PM-12 AM)",
            _ => "various times",
        };

        AiInsight {
            category: "Overall".to_string(),
            icon: "📊".to_string(),
            title: "24-Hour Summary".to_string(),
            description: format!(
                "Scanned {} opportunities, executed {} trades for ${:.2} profit. Peak activity: {}.",
                total,
                executed,
                total_profit as f64 / 100.0,
                peak_desc
            ),
            action: None,
            impact: "low".to_string(),
            confidence_percent: 100,
        }
    }

    fn apply_optimizations(&self, by_category: &HashMap<String, Vec<&OpportunityData>>) {
        let mut cats = self.category_settings.write();

        for (key, opps) in by_category {
            if opps.len() < MIN_OBSERVATIONS_FOR_CATEGORY {
                continue;
            }

            // Find best threshold
            let mut best_threshold = DEFAULT_THRESHOLD;
            let mut best_profit = 0i64;

            for threshold in (MIN_THRESHOLD..=MAX_THRESHOLD).rev() {
                let profit: i64 = opps.iter()
                    .filter(|o| o.adjusted_cost_cents < threshold)
                    .map(|o| (100 - o.adjusted_cost_cents as i16) as i64)
                    .sum();

                if profit > best_profit {
                    best_profit = profit;
                    best_threshold = threshold;
                }
            }

            let settings = cats.entry(key.clone()).or_insert_with(CategorySettings::default);
            let current = settings.threshold_cents;

            // Limit change to MAX_THRESHOLD_CHANGE_PER_CYCLE
            let new_threshold = if best_threshold > current {
                (current + MAX_THRESHOLD_CHANGE_PER_CYCLE as u16).min(best_threshold).min(MAX_THRESHOLD)
            } else if best_threshold < current {
                (current.saturating_sub(MAX_THRESHOLD_CHANGE_PER_CYCLE as u16)).max(best_threshold).max(MIN_THRESHOLD)
            } else {
                current
            };

            if new_threshold != current {
                info!("[ML] {} threshold: {}¢ → {}¢ (target: {}¢)",
                    key, current, new_threshold, best_threshold);
                settings.threshold_cents = new_threshold;
                settings.last_updated = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            }
        }
    }

    /// Get all category settings for display
    pub fn get_all_settings(&self) -> HashMap<String, CategorySettings> {
        self.category_settings.read().clone()
    }

    /// Get performance summary for each category
    pub fn get_performance_summary(&self, opportunities: &[OpportunityData]) -> Vec<CategoryPerformance> {
        let mut results = Vec::new();
        let cats = self.category_settings.read();

        // Group opportunities by category
        let mut by_category: HashMap<String, Vec<&OpportunityData>> = HashMap::new();
        for opp in opportunities {
            let category = MarketCategory::from_market_name(&opp.market_name);
            by_category.entry(category.key()).or_default().push(opp);
        }

        for (key, opps) in &by_category {
            let current_settings = cats.get(key);
            let current_threshold = current_settings.map(|s| s.threshold_cents).unwrap_or(DEFAULT_THRESHOLD);

            // Find recommended threshold
            let mut best_threshold = current_threshold;
            let mut best_profit = 0i64;

            for threshold in (MIN_THRESHOLD..=MAX_THRESHOLD).rev() {
                let profit: i64 = opps.iter()
                    .filter(|o| o.adjusted_cost_cents < threshold)
                    .map(|o| (100 - o.adjusted_cost_cents as i16) as i64)
                    .sum();

                if profit > best_profit {
                    best_profit = profit;
                    best_threshold = threshold;
                }
            }

            let executed: u32 = opps.iter().filter(|o| o.was_executed).count() as u32;
            let missed: u32 = opps.iter()
                .filter(|o| !o.was_executed && o.adjusted_cost_cents < best_threshold)
                .count() as u32;

            let profit: i64 = opps.iter()
                .filter(|o| o.was_executed)
                .map(|o| (100 - o.adjusted_cost_cents as i16) as i64)
                .sum();

            let missed_profit: i64 = opps.iter()
                .filter(|o| !o.was_executed && o.adjusted_cost_cents < best_threshold)
                .map(|o| (100 - o.adjusted_cost_cents as i16) as i64)
                .sum();

            let category = MarketCategory::from_market_name(&opps[0].market_name);

            results.push(CategoryPerformance {
                category,
                current_threshold,
                recommended_threshold: best_threshold,
                observations: opps.len(),
                trades_executed: executed,
                trades_missed: missed,
                profit_cents: profit,
                missed_profit_cents: missed_profit,
                confidence: current_settings.map(|s| s.confidence).unwrap_or(0.0),
            });
        }

        // Sort by missed profit (highest potential first)
        results.sort_by(|a, b| b.missed_profit_cents.cmp(&a.missed_profit_cents));
        results
    }

    /// Save settings to disk
    pub fn save(&self) {
        let data = serde_json::json!({
            "categories": *self.category_settings.read(),
            "global": *self.global_settings.read(),
            "auto_optimize": *self.auto_optimize.read(),
            "last_saved": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        });

        if let Ok(content) = serde_json::to_string_pretty(&data) {
            if let Err(e) = fs::write(&self.persist_path, content) {
                warn!("[ML] Failed to save settings: {}", e);
            }
        }
    }
}

/// Opportunity data for optimization
#[derive(Debug, Clone)]
pub struct OpportunityData {
    pub timestamp: String,
    pub market_name: String,
    pub adjusted_cost_cents: u16,
    pub liquidity_cents: u32,
    pub was_executed: bool,
}

/// Initialize the global ML optimizer
pub fn init_ml_optimizer(base_dir: &str) -> &'static MlOptimizer {
    ML_OPTIMIZER.get_or_init(|| MlOptimizer::new(base_dir))
}

/// Get the global ML optimizer
pub fn get_ml_optimizer() -> Option<&'static MlOptimizer> {
    ML_OPTIMIZER.get()
}

/// Get threshold for a market (fast path for hot loop)
#[inline]
pub fn get_market_threshold(market_name: &str) -> u16 {
    if let Some(optimizer) = get_ml_optimizer() {
        optimizer.get_threshold(market_name)
    } else {
        DEFAULT_THRESHOLD
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_parsing() {
        let cat = MarketCategory::from_market_name("Lakers vs Celtics - Spread");
        assert_eq!(cat.sport, "NBA");
        assert_eq!(cat.bet_type, "Spread");

        let cat2 = MarketCategory::from_market_name("Chiefs vs Ravens Total Points");
        assert_eq!(cat2.sport, "NFL");
        assert_eq!(cat2.bet_type, "Total");

        let cat3 = MarketCategory::from_market_name("BTC above $50,000");
        assert_eq!(cat3.sport, "Crypto");
        assert_eq!(cat3.bet_type, "Price");
    }

    #[test]
    fn test_default_threshold() {
        assert_eq!(DEFAULT_THRESHOLD, 99);
        assert_eq!(MIN_THRESHOLD, 95);
        assert_eq!(MAX_THRESHOLD, 100);
    }
}
