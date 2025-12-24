//! Web-based configuration UI for the arbitrage bot.
//!
//! Provides a REST API and HTML dashboard to adjust bot settings,
//! monitor status, and view historical performance analytics.

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Default web config port
const DEFAULT_PORT: u16 = 8080;

/// Runtime configuration that can be modified via web UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    // === TRADING SETTINGS ===
    /// Arbitrage threshold in cents (e.g., 995 = 99.5¢ total cost = 0.5% profit)
    pub arb_threshold_cents: u16,
    /// Dry run mode - simulate trades without executing
    pub dry_run: bool,
    /// Priority mode enabled
    pub priority_mode: bool,
    /// Crypto markets enabled
    pub crypto_enabled: bool,

    // === LIQUIDITY SETTINGS ===
    /// Minimum liquidity in cents per side
    pub min_liquidity_cents: u32,
    /// Maximum liquidity in cents per side
    pub max_liquidity_cents: u32,

    // === CIRCUIT BREAKER ===
    /// Maximum daily loss in cents before halting
    pub max_daily_loss_cents: u32,
    /// Maximum position size in cents
    pub max_position_size_cents: u32,
    /// Cooldown period in seconds after circuit breaker trips
    pub cooldown_secs: u32,

    // === TIMING ===
    /// Queue sort interval in seconds
    pub queue_sort_interval_secs: u32,
    /// WebSocket reconnect delay in seconds
    pub ws_reconnect_delay_secs: u32,

    // === LEAGUES (restart required) ===
    /// Enabled leagues (comma-separated)
    pub enabled_leagues: String,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            arb_threshold_cents: 995,
            dry_run: true,
            priority_mode: true,
            crypto_enabled: true,
            min_liquidity_cents: 25000,
            max_liquidity_cents: 250000,
            max_daily_loss_cents: 100000,
            max_position_size_cents: 50000,
            cooldown_secs: 300,
            queue_sort_interval_secs: 2,
            ws_reconnect_delay_secs: 2,
            enabled_leagues: String::new(),
        }
    }
}

impl RuntimeConfig {
    /// Load config from .env file
    pub fn from_env() -> Self {
        let get_env = |key: &str, default: &str| -> String {
            std::env::var(key).unwrap_or_else(|_| default.to_string())
        };

        let parse_bool = |val: &str| -> bool {
            val == "1" || val.to_lowercase() == "true"
        };

        Self {
            arb_threshold_cents: get_env("ARB_THRESHOLD_CENTS", "995")
                .parse().unwrap_or(995),
            dry_run: parse_bool(&get_env("DRY_RUN", "1")),
            priority_mode: parse_bool(&get_env("PRIORITY_MODE", "1")),
            crypto_enabled: parse_bool(&get_env("CRYPTO_ENABLED", "0")),
            min_liquidity_cents: get_env("MIN_LIQUIDITY_CENTS", "25000")
                .parse().unwrap_or(25000),
            max_liquidity_cents: get_env("MAX_LIQUIDITY_CENTS", "250000")
                .parse().unwrap_or(250000),
            max_daily_loss_cents: get_env("MAX_DAILY_LOSS_CENTS", "100000")
                .parse().unwrap_or(100000),
            max_position_size_cents: get_env("MAX_POSITION_SIZE_CENTS", "50000")
                .parse().unwrap_or(50000),
            cooldown_secs: get_env("COOLDOWN_SECS", "300")
                .parse().unwrap_or(300),
            queue_sort_interval_secs: get_env("QUEUE_SORT_INTERVAL_SECS", "2")
                .parse().unwrap_or(2),
            ws_reconnect_delay_secs: get_env("WS_RECONNECT_DELAY_SECS", "2")
                .parse().unwrap_or(2),
            enabled_leagues: get_env("ENABLED_LEAGUES", ""),
        }
    }

    /// Save config to .env file (preserves other settings)
    pub fn save_to_env(&self, env_path: &str) -> Result<(), std::io::Error> {
        let existing = std::fs::read_to_string(env_path).unwrap_or_default();
        let mut lines: Vec<String> = existing.lines().map(|s| s.to_string()).collect();

        let updates: HashMap<&str, String> = [
            ("ARB_THRESHOLD_CENTS", self.arb_threshold_cents.to_string()),
            ("DRY_RUN", if self.dry_run { "1" } else { "0" }.to_string()),
            ("PRIORITY_MODE", if self.priority_mode { "1" } else { "0" }.to_string()),
            ("CRYPTO_ENABLED", if self.crypto_enabled { "1" } else { "0" }.to_string()),
            ("MIN_LIQUIDITY_CENTS", self.min_liquidity_cents.to_string()),
            ("MAX_LIQUIDITY_CENTS", self.max_liquidity_cents.to_string()),
            ("MAX_DAILY_LOSS_CENTS", self.max_daily_loss_cents.to_string()),
            ("MAX_POSITION_SIZE_CENTS", self.max_position_size_cents.to_string()),
            ("COOLDOWN_SECS", self.cooldown_secs.to_string()),
            ("QUEUE_SORT_INTERVAL_SECS", self.queue_sort_interval_secs.to_string()),
            ("WS_RECONNECT_DELAY_SECS", self.ws_reconnect_delay_secs.to_string()),
            ("ENABLED_LEAGUES", self.enabled_leagues.clone()),
        ].into_iter().collect();

        let mut found: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for line in lines.iter_mut() {
            for (key, value) in &updates {
                if line.starts_with(&format!("{}=", key)) {
                    *line = format!("{}={}", key, value);
                    found.insert(key);
                    break;
                }
            }
        }

        for (key, value) in &updates {
            if !found.contains(key) {
                lines.push(format!("{}={}", key, value));
            }
        }

        std::fs::write(env_path, lines.join("\n") + "\n")
    }
}

/// Bot status information
#[derive(Debug, Clone, Serialize)]
pub struct BotStatus {
    pub running: bool,
    pub uptime_secs: u64,
    pub uptime_formatted: String,
    pub mode: String,
    pub dry_run: bool,
    pub priority_mode: bool,
    pub crypto_enabled: bool,
    pub markets_tracked: usize,
    pub last_heartbeat_secs_ago: u64,
}

/// Analytics summary from trade log
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalyticsSummary {
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
    pub win_rate_percent: f64,
    pub bankroll_cents: u64,
    pub roi_percent: f64,
    pub uptime_hours: f64,
}

/// Recent trade record for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeDisplay {
    pub id: u64,
    pub timestamp: String,
    pub market_name: String,
    pub arb_type: String,
    pub profit_cents: i64,
    pub volume_cents: u64,
    pub status: String,
    pub latency_ms: f64,
}

/// Open position for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenPositionDisplay {
    pub market_id: String,
    pub description: String,
    pub matched_contracts: f64,
    pub unmatched_contracts: f64,
    pub total_cost_dollars: f64,
    pub guaranteed_profit_dollars: f64,
    pub opened_at: String,
    pub kalshi_yes: f64,
    pub kalshi_no: f64,
    pub poly_yes: f64,
    pub poly_no: f64,
}

/// Shared state for web server
pub struct WebState {
    pub config: RwLock<RuntimeConfig>,
    pub start_time: Instant,
}

pub type SharedWebState = Arc<WebState>;

/// Settings metadata for the UI
#[derive(Serialize)]
struct SettingMeta {
    key: &'static str,
    label: &'static str,
    description: &'static str,
    setting_type: &'static str,
    requires_restart: bool,
    min: Option<u32>,
    max: Option<u32>,
}

fn get_settings_meta() -> Vec<SettingMeta> {
    vec![
        SettingMeta {
            key: "arb_threshold_cents",
            label: "Arb Threshold (cents)",
            description: "Total cost threshold. 995 = 99.5¢ = 0.5% min profit. Lower = more aggressive.",
            setting_type: "number",
            requires_restart: false,
            min: Some(900),
            max: Some(999),
        },
        SettingMeta {
            key: "dry_run",
            label: "Paper Trading",
            description: "Paper mode simulates trades without real money. Turn OFF for live trading.",
            setting_type: "toggle",
            requires_restart: false,
            min: None,
            max: None,
        },
        SettingMeta {
            key: "priority_mode",
            label: "Priority Mode",
            description: "Smart queue sorting by profit, liquidity, and expiration time",
            setting_type: "toggle",
            requires_restart: true,
            min: None,
            max: None,
        },
        SettingMeta {
            key: "crypto_enabled",
            label: "Crypto Markets",
            description: "Enable BTC/ETH crypto market discovery and trading",
            setting_type: "toggle",
            requires_restart: true,
            min: None,
            max: None,
        },
        SettingMeta {
            key: "min_liquidity_cents",
            label: "Min Trade Size ($)",
            description: "Minimum trade size - ignores opportunities smaller than this",
            setting_type: "dollars",
            requires_restart: false,
            min: Some(10),
            max: Some(10000),
        },
        SettingMeta {
            key: "max_daily_loss_cents",
            label: "Max Daily Loss ($)",
            description: "Circuit breaker: halt ALL trading if daily loss exceeds this",
            setting_type: "dollars",
            requires_restart: false,
            min: Some(100),
            max: Some(100000),
        },
        SettingMeta {
            key: "max_position_size_cents",
            label: "Max Position Size ($)",
            description: "Maximum exposure per market (trade size capped by this + liquidity)",
            setting_type: "dollars",
            requires_restart: false,
            min: Some(100),
            max: Some(100000),
        },
        SettingMeta {
            key: "cooldown_secs",
            label: "Cooldown (seconds)",
            description: "Pause duration after circuit breaker trips",
            setting_type: "number",
            requires_restart: false,
            min: Some(60),
            max: Some(3600),
        },
    ]
}

/// GET /api/config
async fn get_config(State(state): State<SharedWebState>) -> impl IntoResponse {
    let cfg = state.config.read().await;
    Json(cfg.clone())
}

/// GET /api/meta
async fn get_meta() -> impl IntoResponse {
    Json(get_settings_meta())
}

/// GET /api/status
async fn get_status(State(state): State<SharedWebState>) -> impl IntoResponse {
    let cfg = state.config.read().await;
    let uptime = state.start_time.elapsed().as_secs();

    let hours = uptime / 3600;
    let mins = (uptime % 3600) / 60;
    let secs = uptime % 60;
    let uptime_formatted = if hours > 0 {
        format!("{}h {}m {}s", hours, mins, secs)
    } else if mins > 0 {
        format!("{}m {}s", mins, secs)
    } else {
        format!("{}s", secs)
    };

    let mode = if cfg.dry_run { "PAPER" } else { "LIVE" };

    let status = BotStatus {
        running: true,
        uptime_secs: uptime,
        uptime_formatted,
        mode: mode.to_string(),
        dry_run: cfg.dry_run,
        priority_mode: cfg.priority_mode,
        crypto_enabled: cfg.crypto_enabled,
        markets_tracked: 0, // Updated by heartbeat
        last_heartbeat_secs_ago: 0,
    };

    Json(status)
}

/// GET /api/analytics
async fn get_analytics() -> impl IntoResponse {
    // Read summary from dashboard_data/summary.json
    let summary_path = "./dashboard_data/summary.json";

    let analytics = if let Ok(content) = std::fs::read_to_string(summary_path) {
        if let Ok(summary) = serde_json::from_str::<serde_json::Value>(&content) {
            let total_trades = summary.get("total_trades").and_then(|v| v.as_u64()).unwrap_or(0);
            let successful = summary.get("successful_trades").and_then(|v| v.as_u64()).unwrap_or(0);
            let profit = summary.get("total_profit_cents").and_then(|v| v.as_i64()).unwrap_or(0);
            let volume = summary.get("total_volume_cents").and_then(|v| v.as_u64()).unwrap_or(0);
            let bankroll = summary.get("bankroll_cents").and_then(|v| v.as_u64()).unwrap_or(100000);

            let win_rate = if total_trades > 0 {
                (successful as f64 / total_trades as f64) * 100.0
            } else { 0.0 };

            let roi = if bankroll > 0 {
                (profit as f64 / bankroll as f64) * 100.0
            } else { 0.0 };

            AnalyticsSummary {
                total_trades,
                successful_trades: successful,
                rejected_trades: summary.get("rejected_trades").and_then(|v| v.as_u64()).unwrap_or(0),
                total_profit_cents: profit,
                total_volume_cents: volume,
                total_fees_cents: summary.get("total_fees_cents").and_then(|v| v.as_u64()).unwrap_or(0),
                avg_profit_per_trade_cents: summary.get("avg_profit_per_trade_cents").and_then(|v| v.as_f64()).unwrap_or(0.0),
                trades_per_hour: summary.get("trades_per_hour").and_then(|v| v.as_f64()).unwrap_or(0.0),
                profit_per_hour_cents: summary.get("profit_per_hour_cents").and_then(|v| v.as_f64()).unwrap_or(0.0),
                best_trade_profit_cents: summary.get("best_trade_profit_cents").and_then(|v| v.as_i64()).unwrap_or(0),
                worst_trade_profit_cents: summary.get("worst_trade_profit_cents").and_then(|v| v.as_i64()).unwrap_or(0),
                avg_latency_ms: summary.get("avg_latency_ms").and_then(|v| v.as_f64()).unwrap_or(0.0),
                win_rate_percent: win_rate,
                bankroll_cents: bankroll,
                roi_percent: roi,
                uptime_hours: 0.0,
            }
        } else {
            AnalyticsSummary::default()
        }
    } else {
        AnalyticsSummary::default()
    };

    Json(analytics)
}

/// GET /api/trades
async fn get_trades() -> impl IntoResponse {
    let trades_path = "./dashboard_data/trades.json";

    let trades: Vec<TradeDisplay> = if let Ok(content) = std::fs::read_to_string(trades_path) {
        if let Ok(all_trades) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
            all_trades.iter().rev().take(50).map(|t| {
                TradeDisplay {
                    id: t.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                    timestamp: t.get("timestamp").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    market_name: t.get("market_name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
                    arb_type: t.get("arb_type").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    profit_cents: t.get("profit_cents").and_then(|v| v.as_i64()).unwrap_or(0),
                    volume_cents: t.get("volume_cents").and_then(|v| v.as_u64()).unwrap_or(0),
                    status: t.get("status").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
                    latency_ms: t.get("latency_ms").and_then(|v| v.as_f64()).unwrap_or(0.0),
                }
            }).collect()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    Json(trades)
}

/// GET /api/positions - Get open positions
async fn get_positions() -> impl IntoResponse {
    let positions_path = "./positions.json";

    let positions: Vec<OpenPositionDisplay> = if let Ok(content) = std::fs::read_to_string(positions_path) {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(positions_map) = data.get("positions").and_then(|v| v.as_object()) {
                positions_map.values().filter_map(|p| {
                    let status = p.get("status").and_then(|v| v.as_str()).unwrap_or("");
                    if status != "open" {
                        return None;
                    }

                    let kalshi_yes = p.get("kalshi_yes").and_then(|v| v.get("contracts")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let kalshi_no = p.get("kalshi_no").and_then(|v| v.get("contracts")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let poly_yes = p.get("poly_yes").and_then(|v| v.get("contracts")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let poly_no = p.get("poly_no").and_then(|v| v.get("contracts")).and_then(|v| v.as_f64()).unwrap_or(0.0);

                    let yes_total = kalshi_yes + poly_yes;
                    let no_total = kalshi_no + poly_no;
                    let matched = yes_total.min(no_total);
                    let unmatched = (yes_total - no_total).abs();

                    let kalshi_yes_cost = p.get("kalshi_yes").and_then(|v| v.get("cost_basis")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let kalshi_no_cost = p.get("kalshi_no").and_then(|v| v.get("cost_basis")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let poly_yes_cost = p.get("poly_yes").and_then(|v| v.get("cost_basis")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let poly_no_cost = p.get("poly_no").and_then(|v| v.get("cost_basis")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let fees = p.get("total_fees").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let total_cost = kalshi_yes_cost + kalshi_no_cost + poly_yes_cost + poly_no_cost + fees;

                    // Guaranteed profit = matched contracts * $1 - total cost
                    let guaranteed_profit = matched - total_cost;

                    Some(OpenPositionDisplay {
                        market_id: p.get("market_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        description: p.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        matched_contracts: matched,
                        unmatched_contracts: unmatched,
                        total_cost_dollars: total_cost,
                        guaranteed_profit_dollars: guaranteed_profit,
                        opened_at: p.get("opened_at").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        kalshi_yes,
                        kalshi_no,
                        poly_yes,
                        poly_no,
                    })
                }).collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    Json(positions)
}

/// POST /api/config
async fn update_config(
    State(state): State<SharedWebState>,
    Json(new_config): Json<RuntimeConfig>,
) -> impl IntoResponse {
    if let Err(e) = new_config.save_to_env(".env") {
        warn!("[WEB] Failed to save config: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save config");
    }

    let mut cfg = state.config.write().await;
    *cfg = new_config;

    info!("[WEB] Config updated successfully");
    (StatusCode::OK, "Config saved")
}

/// POST /api/restart
async fn trigger_restart() -> impl IntoResponse {
    info!("[WEB] Restart requested via web UI - shutting down in 2 seconds");

    // Spawn a task to exit after a short delay (allows response to be sent)
    tokio::spawn(async {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        info!("[WEB] Exiting for restart...");
        std::process::exit(0);
    });

    (StatusCode::OK, "Bot shutting down in 2 seconds. Use run_bot.sh for auto-restart.")
}

/// GET /
async fn serve_dashboard() -> Html<&'static str> {
    Html(DASHBOARD_HTML)
}

/// PWA manifest for mobile "Add to Home Screen"
async fn serve_manifest() -> impl IntoResponse {
    let manifest = r##"{
    "name": "Arbitrage Bot Dashboard",
    "short_name": "Arb Bot",
    "description": "Real-time arbitrage trading bot monitoring",
    "start_url": "/",
    "display": "standalone",
    "background_color": "#0d1117",
    "theme_color": "#0d1117",
    "orientation": "portrait-primary",
    "icons": [
        {
            "src": "/icon.svg",
            "sizes": "any",
            "type": "image/svg+xml",
            "purpose": "any"
        }
    ]
}"##;
    (
        StatusCode::OK,
        [("Content-Type", "application/manifest+json")],
        manifest
    )
}

/// Simple SVG icon for PWA
async fn serve_icon() -> impl IntoResponse {
    // Simple green dollar sign icon as SVG
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="192" height="192" viewBox="0 0 192 192">
        <rect width="192" height="192" rx="32" fill="#0d1117"/>
        <circle cx="96" cy="96" r="70" fill="#238636" opacity="0.2"/>
        <path d="M96 40 L96 152 M76 60 L116 60 M76 132 L116 132"
              stroke="#3fb950" stroke-width="8" stroke-linecap="round"/>
        <path d="M70 80 Q70 60 96 60 Q122 60 122 80 Q122 96 96 96 Q70 96 70 112 Q70 132 96 132 Q122 132 122 112"
              stroke="#3fb950" stroke-width="8" fill="none" stroke-linecap="round"/>
    </svg>"##;
    (
        StatusCode::OK,
        [("Content-Type", "image/svg+xml")],
        svg
    )
}

/// Create and run the web server
pub async fn run_web_server(config: Arc<RwLock<RuntimeConfig>>) {
    let port = std::env::var("WEB_CONFIG_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    let state = Arc::new(WebState {
        config: RwLock::new(config.read().await.clone()),
        start_time: Instant::now(),
    });

    let app = Router::new()
        .route("/", get(serve_dashboard))
        .route("/manifest.json", get(serve_manifest))
        .route("/icon.svg", get(serve_icon))
        .route("/api/config", get(get_config).post(update_config))
        .route("/api/meta", get(get_meta))
        .route("/api/status", get(get_status))
        .route("/api/analytics", get(get_analytics))
        .route("/api/trades", get(get_trades))
        .route("/api/positions", get(get_positions))
        .route("/api/restart", post(trigger_restart))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    info!("[WEB] Starting config dashboard on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    if let Err(e) = axum::serve(listener, app).await {
        warn!("[WEB] Server error: {}", e);
    }
}

/// Embedded HTML dashboard with status bar and analytics
const DASHBOARD_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Arb Bot Dashboard</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #0d1117;
            color: #c9d1d9;
            padding: 20px;
            max-width: 1200px;
            margin: 0 auto;
        }
        h1 { color: #58a6ff; margin-bottom: 8px; font-size: 24px; }
        .subtitle { color: #8b949e; margin-bottom: 16px; font-size: 14px; }

        /* Status Bar */
        .status-bar {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
            gap: 12px;
            margin-bottom: 20px;
        }
        .status-card {
            background: #161b22;
            border: 1px solid #30363d;
            border-radius: 8px;
            padding: 16px;
            text-align: center;
        }
        .status-card.live { border-color: #f85149; }
        .status-card.dry { border-color: #238636; }
        .status-value {
            font-size: 24px;
            font-weight: 700;
            color: #f0f6fc;
        }
        .status-value.positive { color: #3fb950; }
        .status-value.negative { color: #f85149; }
        .status-value.warning { color: #d29922; }
        .status-label {
            font-size: 12px;
            color: #8b949e;
            margin-top: 4px;
            text-transform: uppercase;
        }
        .mode-badge {
            display: inline-block;
            padding: 4px 12px;
            border-radius: 20px;
            font-size: 14px;
            font-weight: 600;
        }
        .mode-badge.live { background: #f85149; color: white; }
        .mode-badge.dry { background: #238636; color: white; }

        /* Tabs */
        .tabs {
            display: flex;
            gap: 8px;
            margin-bottom: 20px;
            border-bottom: 1px solid #30363d;
            padding-bottom: 8px;
        }
        .tab {
            padding: 8px 16px;
            background: transparent;
            border: none;
            color: #8b949e;
            cursor: pointer;
            font-size: 14px;
            border-radius: 6px;
        }
        .tab.active { background: #21262d; color: #f0f6fc; }
        .tab:hover { color: #f0f6fc; }

        /* Tab content */
        .tab-content { display: none; }
        .tab-content.active { display: block; }

        /* Sections */
        .section {
            background: #161b22;
            border: 1px solid #30363d;
            border-radius: 8px;
            padding: 20px;
            margin-bottom: 16px;
        }
        .section-title {
            color: #f0f6fc;
            font-size: 16px;
            font-weight: 600;
            margin-bottom: 16px;
            padding-bottom: 8px;
            border-bottom: 1px solid #30363d;
        }

        /* Analytics Grid */
        .analytics-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 16px;
        }
        .metric-card {
            background: #0d1117;
            border-radius: 6px;
            padding: 16px;
        }
        .metric-value {
            font-size: 28px;
            font-weight: 700;
            color: #f0f6fc;
        }
        .metric-label {
            font-size: 12px;
            color: #8b949e;
            margin-top: 4px;
        }

        /* Settings */
        .setting {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 12px 0;
            border-bottom: 1px solid #21262d;
        }
        .setting:last-child { border-bottom: none; }
        .setting-info { flex: 1; }
        .setting-label {
            color: #f0f6fc;
            font-weight: 500;
            display: flex;
            align-items: center;
            gap: 8px;
        }
        .restart-badge {
            background: #f85149;
            color: white;
            font-size: 10px;
            padding: 2px 6px;
            border-radius: 10px;
            font-weight: 600;
        }
        .setting-desc {
            color: #8b949e;
            font-size: 12px;
            margin-top: 4px;
        }
        .setting-control { min-width: 150px; text-align: right; }

        input[type="number"], input[type="text"] {
            background: #0d1117;
            border: 1px solid #30363d;
            border-radius: 6px;
            color: #c9d1d9;
            padding: 8px 12px;
            width: 120px;
            font-size: 14px;
        }
        input:focus { border-color: #58a6ff; outline: none; }

        /* Toggle Button - Fixed styling */
        .toggle-btn {
            display: inline-flex;
            align-items: center;
            gap: 8px;
            cursor: pointer;
            user-select: none;
        }
        .toggle-btn input { display: none; }
        .toggle-switch {
            width: 48px;
            height: 24px;
            background: #f85149;
            border-radius: 12px;
            position: relative;
            transition: background 0.3s;
            flex-shrink: 0;
        }
        .toggle-switch:after {
            content: "";
            position: absolute;
            width: 20px;
            height: 20px;
            background: white;
            border-radius: 50%;
            top: 2px;
            left: 2px;
            transition: transform 0.3s;
        }
        .toggle-btn input:checked + .toggle-switch {
            background: #238636;
        }
        .toggle-btn input:checked + .toggle-switch:after {
            transform: translateX(24px);
        }
        .toggle-label {
            font-size: 12px;
            color: #8b949e;
            min-width: 40px;
        }
        .toggle-btn input:checked ~ .toggle-label { color: #3fb950; }
        .toggle-btn input:not(:checked) ~ .toggle-label { color: #f85149; }

        /* Buttons */
        .btn {
            background: #238636;
            color: white;
            border: none;
            padding: 10px 20px;
            border-radius: 6px;
            font-size: 14px;
            font-weight: 600;
            cursor: pointer;
            margin-right: 8px;
        }
        .btn:hover { background: #2ea043; }
        .btn-secondary { background: #30363d; }
        .btn-secondary:hover { background: #3d444d; }
        .btn-danger { background: #da3633; }
        .btn-danger:hover { background: #f85149; }
        .actions { margin-top: 20px; display: flex; gap: 12px; }

        /* Trade Table */
        .trade-table {
            width: 100%;
            border-collapse: collapse;
            font-size: 13px;
        }
        .trade-table th {
            text-align: left;
            padding: 12px 8px;
            border-bottom: 1px solid #30363d;
            color: #8b949e;
            font-weight: 500;
        }
        .trade-table td {
            padding: 10px 8px;
            border-bottom: 1px solid #21262d;
        }
        .trade-table tr:hover { background: #21262d; }
        .profit-positive { color: #3fb950; }
        .profit-negative { color: #f85149; }
        .status-executed { color: #3fb950; }
        .status-dryrun { color: #58a6ff; }
        .status-rejected { color: #f85149; }

        /* Status indicators */
        .status-dot {
            display: inline-block;
            width: 8px;
            height: 8px;
            border-radius: 50%;
            margin-right: 6px;
        }
        .status-dot.green { background: #3fb950; }
        .status-dot.red { background: #f85149; }
        .status-dot.yellow { background: #d29922; }

        .status-msg {
            padding: 12px;
            border-radius: 6px;
            margin-top: 16px;
            display: none;
        }
        .status-msg.success { display: block; background: #238636; color: white; }
        .status-msg.error { display: block; background: #da3633; color: white; }
        .status-msg.warning { display: block; background: #9e6a03; color: white; }

        .loader { display: none; color: #8b949e; padding: 20px; text-align: center; }
        #content { display: none; }

        /* Mobile Responsive */
        @media (max-width: 768px) {
            body { padding: 12px; }
            h1 { font-size: 20px; }
            .subtitle { font-size: 12px; margin-bottom: 12px; }

            /* Status bar - 2 columns on mobile */
            .status-bar {
                grid-template-columns: repeat(2, 1fr);
                gap: 8px;
            }
            .status-card { padding: 12px; }
            .status-value { font-size: 18px; }
            .status-label { font-size: 10px; }

            /* Tabs - horizontal scroll */
            .tabs {
                overflow-x: auto;
                -webkit-overflow-scrolling: touch;
                scrollbar-width: none;
                padding-bottom: 12px;
            }
            .tabs::-webkit-scrollbar { display: none; }
            .tab {
                flex-shrink: 0;
                padding: 10px 14px;
                font-size: 13px;
            }

            /* Sections */
            .section { padding: 14px; }
            .section-title { font-size: 14px; }

            /* Settings */
            .setting {
                flex-direction: column;
                align-items: flex-start;
                gap: 10px;
                padding: 14px 0;
            }
            .setting-info { width: 100%; }
            .setting-control {
                width: 100%;
                text-align: left;
            }
            .setting-control input[type="number"] {
                width: 100%;
                padding: 12px;
                font-size: 16px; /* Prevents iOS zoom */
            }

            /* Toggle buttons - larger touch targets */
            .toggle-switch {
                width: 52px;
                height: 28px;
            }
            .toggle-switch:after {
                width: 22px;
                height: 22px;
                top: 3px;
                left: 3px;
            }
            .toggle-btn input:checked + .toggle-switch:after {
                transform: translateX(24px);
            }

            /* Action buttons - full width */
            .actions {
                flex-direction: column;
                gap: 10px;
            }
            .btn {
                width: 100%;
                padding: 14px 20px;
                font-size: 15px;
                margin-right: 0;
                text-align: center;
            }

            /* Analytics grid */
            .analytics-grid {
                grid-template-columns: repeat(2, 1fr);
                gap: 10px;
            }
            .metric-card { padding: 12px; }
            .metric-value { font-size: 18px; }
            .metric-label { font-size: 10px; }

            /* Trade table - horizontal scroll */
            .section:has(.trade-table) {
                padding: 14px 0;
            }
            .trade-table-wrapper {
                overflow-x: auto;
                -webkit-overflow-scrolling: touch;
                margin: 0 -14px;
                padding: 0 14px;
            }
            .trade-table {
                min-width: 600px;
                font-size: 12px;
            }
            .trade-table th, .trade-table td {
                padding: 8px 6px;
                white-space: nowrap;
            }

            /* Positions table */
            .positions-grid {
                grid-template-columns: 1fr;
            }
        }

        /* Extra small screens */
        @media (max-width: 380px) {
            .status-bar { grid-template-columns: 1fr; }
            .analytics-grid { grid-template-columns: 1fr; }
        }

        /* PWA install prompt */
        .install-prompt {
            display: none;
            position: fixed;
            bottom: 20px;
            left: 20px;
            right: 20px;
            background: #238636;
            color: white;
            padding: 16px;
            border-radius: 12px;
            box-shadow: 0 4px 20px rgba(0,0,0,0.4);
            z-index: 1000;
            text-align: center;
        }
        .install-prompt.show { display: block; }
        .install-prompt button {
            background: white;
            color: #238636;
            border: none;
            padding: 10px 20px;
            border-radius: 6px;
            font-weight: 600;
            margin-top: 10px;
            cursor: pointer;
        }
        .install-close {
            position: absolute;
            top: 8px;
            right: 12px;
            background: none;
            border: none;
            color: white;
            font-size: 20px;
            cursor: pointer;
        }
    </style>

    <!-- PWA Support -->
    <link rel="manifest" href="/manifest.json">
    <link rel="icon" href="/icon.svg" type="image/svg+xml">
    <link rel="apple-touch-icon" href="/icon.svg">
    <meta name="apple-mobile-web-app-capable" content="yes">
    <meta name="apple-mobile-web-app-status-bar-style" content="black-translucent">
    <meta name="apple-mobile-web-app-title" content="Arb Bot">
    <meta name="theme-color" content="#0d1117">
</head>
<body>
    <h1>Arbitrage Bot Dashboard</h1>
    <p class="subtitle">Real-time monitoring and configuration</p>

    <div class="loader" id="loader">Loading...</div>
    <div id="status-msg" class="status-msg"></div>

    <div id="content">
        <!-- Status Bar -->
        <div class="status-bar" id="status-bar">
            <div class="status-card" id="mode-card">
                <div class="status-value"><span class="mode-badge dry" id="mode-badge">PAPER</span></div>
                <div class="status-label">Trading Mode</div>
            </div>
            <div class="status-card">
                <div class="status-value" id="uptime">0s</div>
                <div class="status-label">Uptime</div>
            </div>
            <div class="status-card">
                <div class="status-value" id="total-profit">$0.00</div>
                <div class="status-label">Total Profit</div>
            </div>
            <div class="status-card">
                <div class="status-value" id="total-trades">0</div>
                <div class="status-label">Total Trades</div>
            </div>
            <div class="status-card">
                <div class="status-value" id="win-rate">0%</div>
                <div class="status-label">Win Rate</div>
            </div>
            <div class="status-card">
                <div class="status-value" id="roi">0%</div>
                <div class="status-label">ROI</div>
            </div>
        </div>

        <!-- Tabs -->
        <div class="tabs">
            <button class="tab active" onclick="showTab('analytics')">Analytics</button>
            <button class="tab" onclick="showTab('positions')">Open Positions</button>
            <button class="tab" onclick="showTab('settings')">Settings</button>
            <button class="tab" onclick="showTab('trades')">Trade History</button>
        </div>

        <!-- Analytics Tab -->
        <div id="tab-analytics" class="tab-content active">
            <div class="section">
                <div class="section-title">Performance Metrics</div>
                <div class="analytics-grid">
                    <div class="metric-card">
                        <div class="metric-value" id="profit-hour">$0.00</div>
                        <div class="metric-label">Profit / Hour</div>
                    </div>
                    <div class="metric-card">
                        <div class="metric-value" id="trades-hour">0</div>
                        <div class="metric-label">Trades / Hour</div>
                    </div>
                    <div class="metric-card">
                        <div class="metric-value" id="avg-profit">$0.00</div>
                        <div class="metric-label">Avg Profit / Trade</div>
                    </div>
                    <div class="metric-card">
                        <div class="metric-value" id="avg-latency">0ms</div>
                        <div class="metric-label">Avg Latency</div>
                    </div>
                    <div class="metric-card">
                        <div class="metric-value" id="best-trade">$0.00</div>
                        <div class="metric-label">Best Trade</div>
                    </div>
                    <div class="metric-card">
                        <div class="metric-value" id="total-volume">$0</div>
                        <div class="metric-label">Total Volume</div>
                    </div>
                </div>
            </div>
            <div class="section">
                <div class="section-title">30-Day Projection (Based on Current Rate)</div>
                <div class="analytics-grid">
                    <div class="metric-card">
                        <div class="metric-value positive" id="proj-profit">$0.00</div>
                        <div class="metric-label">Projected Profit</div>
                    </div>
                    <div class="metric-card">
                        <div class="metric-value" id="proj-trades">0</div>
                        <div class="metric-label">Projected Trades</div>
                    </div>
                    <div class="metric-card">
                        <div class="metric-value" id="proj-volume">$0</div>
                        <div class="metric-label">Projected Volume</div>
                    </div>
                    <div class="metric-card">
                        <div class="metric-value positive" id="proj-roi">0%</div>
                        <div class="metric-label">Projected 30d ROI</div>
                    </div>
                </div>
            </div>
        </div>

        <!-- Open Positions Tab -->
        <div id="tab-positions" class="tab-content">
            <div class="section">
                <div class="section-title">Open Positions (Awaiting Settlement)</div>
                <div id="positions-summary" style="display:grid;grid-template-columns:repeat(4,1fr);gap:12px;margin-bottom:16px;">
                    <div class="metric-card">
                        <div class="metric-value" id="pos-count">0</div>
                        <div class="metric-label">Open Positions</div>
                    </div>
                    <div class="metric-card">
                        <div class="metric-value" id="pos-matched">0</div>
                        <div class="metric-label">Matched Contracts</div>
                    </div>
                    <div class="metric-card">
                        <div class="metric-value positive" id="pos-profit">$0.00</div>
                        <div class="metric-label">Guaranteed Profit</div>
                    </div>
                    <div class="metric-card">
                        <div class="metric-value warning" id="pos-unmatched">0</div>
                        <div class="metric-label">Unmatched (Risk)</div>
                    </div>
                </div>
                <table class="trade-table">
                    <thead>
                        <tr>
                            <th>Market</th>
                            <th>Matched</th>
                            <th>Cost</th>
                            <th>Profit</th>
                            <th>K-Yes</th>
                            <th>K-No</th>
                            <th>P-Yes</th>
                            <th>P-No</th>
                            <th>Opened</th>
                        </tr>
                    </thead>
                    <tbody id="positions-body">
                        <tr><td colspan="9" style="text-align:center;color:#8b949e">No open positions</td></tr>
                    </tbody>
                </table>
                <p style="margin-top:12px;color:#8b949e;font-size:12px;">
                    ℹ️ <strong>Matched positions</strong> = guaranteed profit regardless of outcome.
                    <strong>Unmatched</strong> = partial fills with market risk (bot auto-closes these).
                </p>
            </div>
        </div>

        <!-- Settings Tab -->
        <div id="tab-settings" class="tab-content">
            <div class="section">
                <div class="section-title">Trading Controls</div>
                <div id="trading-settings"></div>
            </div>

            <div class="section">
                <div class="section-title">Risk Management</div>
                <div id="circuit-settings"></div>
            </div>

            <div class="actions">
                <button class="btn" onclick="saveConfig()">Save Changes</button>
                <button class="btn btn-secondary" onclick="loadConfig()">Reset</button>
                <button class="btn btn-danger" onclick="restartBot()">Restart Bot</button>
            </div>
        </div>

        <!-- Trades Tab -->
        <div id="tab-trades" class="tab-content">
            <div class="section">
                <div class="section-title">Recent Trades (Last 50)</div>
                <table class="trade-table">
                    <thead>
                        <tr>
                            <th>Time</th>
                            <th>Market</th>
                            <th>Type</th>
                            <th>Profit</th>
                            <th>Volume</th>
                            <th>Latency</th>
                            <th>Status</th>
                        </tr>
                    </thead>
                    <tbody id="trades-body">
                        <tr><td colspan="7" style="text-align:center;color:#8b949e">No trades yet</td></tr>
                    </tbody>
                </table>
            </div>
        </div>
    </div>

    <script>
        let config = {};
        let meta = [];

        const sections = {
            'arb_threshold_cents': 'trading-settings',
            'dry_run': 'trading-settings',
            'priority_mode': 'trading-settings',
            'crypto_enabled': 'trading-settings',
            'min_liquidity_cents': 'circuit-settings',
            'max_daily_loss_cents': 'circuit-settings',
            'max_position_size_cents': 'circuit-settings',
            'cooldown_secs': 'circuit-settings',
        };

        function showTab(name) {
            document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
            document.querySelectorAll('.tab-content').forEach(t => t.classList.remove('active'));
            document.querySelector(`[onclick="showTab('${name}')"]`).classList.add('active');
            document.getElementById('tab-' + name).classList.add('active');
        }

        function formatCents(cents) {
            return '$' + (cents / 100).toFixed(2);
        }

        async function loadAll() {
            document.getElementById('loader').style.display = 'block';
            document.getElementById('content').style.display = 'none';

            try {
                const [configRes, metaRes, statusRes, analyticsRes, tradesRes, positionsRes] = await Promise.all([
                    fetch('/api/config'),
                    fetch('/api/meta'),
                    fetch('/api/status'),
                    fetch('/api/analytics'),
                    fetch('/api/trades'),
                    fetch('/api/positions')
                ]);

                config = await configRes.json();
                meta = await metaRes.json();
                const status = await statusRes.json();
                const analytics = await analyticsRes.json();
                const trades = await tradesRes.json();
                const positions = await positionsRes.json();

                updateStatus(status);
                updateAnalytics(analytics);
                updateTrades(trades);
                updatePositions(positions);
                renderSettings();

                document.getElementById('content').style.display = 'block';
            } catch (e) {
                showStatus('Failed to load: ' + e.message, 'error');
            }
            document.getElementById('loader').style.display = 'none';
        }

        function updateStatus(status) {
            document.getElementById('uptime').textContent = status.uptime_formatted;

            const modeBadge = document.getElementById('mode-badge');
            const modeCard = document.getElementById('mode-card');
            if (status.dry_run) {
                modeBadge.textContent = 'PAPER';
                modeBadge.className = 'mode-badge dry';
                modeCard.className = 'status-card dry';
            } else {
                modeBadge.textContent = 'LIVE';
                modeBadge.className = 'mode-badge live';
                modeCard.className = 'status-card live';
            }
        }

        function updateAnalytics(a) {
            const profitEl = document.getElementById('total-profit');
            profitEl.textContent = formatCents(a.total_profit_cents);
            profitEl.className = 'status-value ' + (a.total_profit_cents >= 0 ? 'positive' : 'negative');

            document.getElementById('total-trades').textContent = a.total_trades;
            document.getElementById('win-rate').textContent = a.win_rate_percent.toFixed(1) + '%';

            const roiEl = document.getElementById('roi');
            roiEl.textContent = a.roi_percent.toFixed(2) + '%';
            roiEl.className = 'status-value ' + (a.roi_percent >= 0 ? 'positive' : 'negative');

            document.getElementById('profit-hour').textContent = formatCents(a.profit_per_hour_cents);
            document.getElementById('trades-hour').textContent = a.trades_per_hour.toFixed(1);
            document.getElementById('avg-profit').textContent = formatCents(a.avg_profit_per_trade_cents);

            // Fix latency display - convert if stored as nanoseconds (> 60000ms is unreasonable)
            let latencyMs = a.avg_latency_ms;
            if (latencyMs > 60000) {
                latencyMs = latencyMs / 1000000; // Convert ns to ms
            }
            document.getElementById('avg-latency').textContent = latencyMs.toFixed(1) + 'ms';

            document.getElementById('best-trade').textContent = formatCents(a.best_trade_profit_cents);
            document.getElementById('total-volume').textContent = formatCents(a.total_volume_cents);

            // 30-day projections based on hourly rates
            const hoursIn30Days = 30 * 24;
            const projProfit = a.profit_per_hour_cents * hoursIn30Days;
            const projTrades = Math.round(a.trades_per_hour * hoursIn30Days);
            const projVolume = (a.total_volume_cents / Math.max(a.total_trades, 1)) * projTrades;
            const projRoi = a.bankroll_cents > 0 ? (projProfit / a.bankroll_cents) * 100 : 0;

            document.getElementById('proj-profit').textContent = formatCents(projProfit);
            document.getElementById('proj-trades').textContent = projTrades.toLocaleString();
            document.getElementById('proj-volume').textContent = formatCents(projVolume);
            document.getElementById('proj-roi').textContent = projRoi.toFixed(1) + '%';
        }

        function updateTrades(trades) {
            const tbody = document.getElementById('trades-body');
            if (!trades.length) {
                tbody.innerHTML = '<tr><td colspan="7" style="text-align:center;color:#8b949e">No trades yet</td></tr>';
                return;
            }

            tbody.innerHTML = trades.map(t => {
                // Format time in US Central without seconds
                const date = new Date(t.timestamp);
                const time = date.toLocaleTimeString('en-US', {
                    timeZone: 'America/Chicago',
                    hour: 'numeric',
                    minute: '2-digit',
                    hour12: true
                });

                // Fix latency - convert ns to ms if too large
                let latencyMs = t.latency_ms;
                if (latencyMs > 60000) {
                    latencyMs = latencyMs / 1000000;
                }

                const profitClass = t.profit_cents >= 0 ? 'profit-positive' : 'profit-negative';
                const statusClass = t.status === 'executed' ? 'status-executed' :
                                   t.status === 'dryrun' ? 'status-dryrun' : 'status-rejected';
                return `<tr>
                    <td>${time}</td>
                    <td>${t.market_name.substring(0, 30)}</td>
                    <td>${t.arb_type}</td>
                    <td class="${profitClass}">${formatCents(t.profit_cents)}</td>
                    <td>${formatCents(t.volume_cents)}</td>
                    <td>${latencyMs.toFixed(1)}ms</td>
                    <td class="${statusClass}">${t.status}</td>
                </tr>`;
            }).join('');
        }

        function updatePositions(positions) {
            const tbody = document.getElementById('positions-body');

            // Update summary
            let totalMatched = 0, totalProfit = 0, totalUnmatched = 0;
            positions.forEach(p => {
                totalMatched += p.matched_contracts;
                totalProfit += p.guaranteed_profit_dollars;
                totalUnmatched += p.unmatched_contracts;
            });

            document.getElementById('pos-count').textContent = positions.length;
            document.getElementById('pos-matched').textContent = totalMatched.toFixed(0);
            document.getElementById('pos-profit').textContent = '$' + totalProfit.toFixed(2);
            document.getElementById('pos-unmatched').textContent = totalUnmatched.toFixed(0);

            if (!positions.length) {
                tbody.innerHTML = '<tr><td colspan="9" style="text-align:center;color:#8b949e">No open positions</td></tr>';
                return;
            }

            tbody.innerHTML = positions.map(p => {
                const profitClass = p.guaranteed_profit_dollars >= 0 ? 'profit-positive' : 'profit-negative';
                const opened = new Date(p.opened_at).toLocaleString();
                return `<tr>
                    <td title="${p.market_id}">${p.description.substring(0, 25)}...</td>
                    <td>${p.matched_contracts.toFixed(0)}</td>
                    <td>$${p.total_cost_dollars.toFixed(2)}</td>
                    <td class="${profitClass}">$${p.guaranteed_profit_dollars.toFixed(2)}</td>
                    <td>${p.kalshi_yes > 0 ? p.kalshi_yes.toFixed(0) : '-'}</td>
                    <td>${p.kalshi_no > 0 ? p.kalshi_no.toFixed(0) : '-'}</td>
                    <td>${p.poly_yes > 0 ? p.poly_yes.toFixed(0) : '-'}</td>
                    <td>${p.poly_no > 0 ? p.poly_no.toFixed(0) : '-'}</td>
                    <td>${opened}</td>
                </tr>`;
            }).join('');
        }

        function renderSettings() {
            Object.values(sections).forEach(id => {
                const el = document.getElementById(id);
                if (el) el.innerHTML = '';
            });

            meta.forEach(setting => {
                const section = document.getElementById(sections[setting.key]);
                if (!section) return;

                const div = document.createElement('div');
                div.className = 'setting';

                let control = '';
                const value = config[setting.key];

                if (setting.setting_type === 'toggle') {
                    const labelOn = setting.key === 'dry_run' ? 'PAPER' : 'ON';
                    const labelOff = setting.key === 'dry_run' ? 'LIVE' : 'OFF';
                    control = `
                        <label class="toggle-btn">
                            <input type="checkbox" id="${setting.key}" ${value ? 'checked' : ''}>
                            <span class="toggle-switch"></span>
                            <span class="toggle-label">${value ? labelOn : labelOff}</span>
                        </label>
                    `;
                } else if (setting.setting_type === 'dollars') {
                    // Display in dollars but store in cents
                    const dollarValue = (value / 100).toFixed(0);
                    control = `
                        <div style="display:flex;align-items:center;gap:4px;">
                            <span style="color:#8b949e;">$</span>
                            <input type="number" id="${setting.key}" value="${dollarValue}"
                                data-type="dollars"
                                ${setting.min !== null ? 'min="' + setting.min + '"' : ''}
                                ${setting.max !== null ? 'max="' + setting.max + '"' : ''}>
                        </div>
                    `;
                } else if (setting.setting_type === 'number') {
                    control = `
                        <input type="number" id="${setting.key}" value="${value}"
                            ${setting.min !== null ? 'min="' + setting.min + '"' : ''}
                            ${setting.max !== null ? 'max="' + setting.max + '"' : ''}>
                    `;
                } else {
                    control = `<input type="text" id="${setting.key}" value="${value || ''}" style="width:200px">`;
                }

                div.innerHTML = `
                    <div class="setting-info">
                        <div class="setting-label">
                            ${setting.label}
                            ${setting.requires_restart ? '<span class="restart-badge">RESTART</span>' : ''}
                        </div>
                        <div class="setting-desc">${setting.description}</div>
                    </div>
                    <div class="setting-control">${control}</div>
                `;
                section.appendChild(div);

                // Add change listener for toggle labels
                if (setting.setting_type === 'toggle') {
                    const checkbox = div.querySelector('input[type="checkbox"]');
                    const label = div.querySelector('.toggle-label');
                    const labelOn = setting.key === 'dry_run' ? 'PAPER' : 'ON';
                    const labelOff = setting.key === 'dry_run' ? 'LIVE' : 'OFF';
                    checkbox.addEventListener('change', () => {
                        label.textContent = checkbox.checked ? labelOn : labelOff;
                    });
                }
            });
        }

        async function saveConfig() {
            // Start with existing config to preserve hidden fields
            const newConfig = { ...config };

            meta.forEach(setting => {
                const el = document.getElementById(setting.key);
                if (!el) return;

                if (setting.setting_type === 'toggle') {
                    newConfig[setting.key] = el.checked;
                } else if (setting.setting_type === 'dollars') {
                    // Convert dollars to cents for storage
                    newConfig[setting.key] = (parseInt(el.value) || 0) * 100;
                } else if (setting.setting_type === 'number') {
                    newConfig[setting.key] = parseInt(el.value) || 0;
                } else {
                    newConfig[setting.key] = el.value;
                }
            });

            try {
                const res = await fetch('/api/config', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(newConfig)
                });

                if (res.ok) {
                    config = newConfig;
                    showStatus('Config saved! Some changes may require a restart.', 'success');
                    loadAll(); // Refresh status
                } else {
                    showStatus('Failed to save config', 'error');
                }
            } catch (e) {
                showStatus('Error: ' + e.message, 'error');
            }
        }

        async function restartBot() {
            if (!confirm('Restart the bot? This will interrupt any active trading.')) return;

            try {
                await fetch('/api/restart', { method: 'POST' });
                showStatus('Restart signal sent. Please restart the bot manually via SSH.', 'warning');
            } catch (e) {
                showStatus('Error: ' + e.message, 'error');
            }
        }

        function showStatus(msg, type) {
            const el = document.getElementById('status-msg');
            el.textContent = msg;
            el.className = 'status-msg ' + type;
            setTimeout(() => { el.className = 'status-msg'; }, 5000);
        }

        async function loadConfig() {
            await loadAll();
        }

        // Initial load and auto-refresh
        loadAll();
        setInterval(async () => {
            try {
                const [statusRes, analyticsRes, positionsRes] = await Promise.all([
                    fetch('/api/status'),
                    fetch('/api/analytics'),
                    fetch('/api/positions')
                ]);
                updateStatus(await statusRes.json());
                updateAnalytics(await analyticsRes.json());
                updatePositions(await positionsRes.json());
            } catch (e) {}
        }, 5000); // Refresh every 5 seconds
    </script>
</body>
</html>
"##;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = RuntimeConfig::default();
        assert_eq!(cfg.arb_threshold_cents, 995);
        assert!(cfg.dry_run);
    }

    #[test]
    fn test_settings_meta() {
        let meta = get_settings_meta();
        assert!(!meta.is_empty());
        assert!(meta.iter().any(|m| m.key == "dry_run"));
    }
}
