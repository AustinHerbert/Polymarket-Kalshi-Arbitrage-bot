//! Web-based configuration UI for the arbitrage bot.
//!
//! Provides a REST API and HTML dashboard to adjust bot settings
//! without SSH access. Runs on port 8080 by default.

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
            crypto_enabled: false,
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
        // Read existing .env content
        let existing = std::fs::read_to_string(env_path).unwrap_or_default();
        let mut lines: Vec<String> = existing.lines().map(|s| s.to_string()).collect();

        // Settings to update
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

        // Update existing lines or mark for addition
        let mut found: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for line in lines.iter_mut() {
            for (key, value) in &updates {
                if line.starts_with(&format!("{}=", key)) || line.starts_with(&format!("{}=", key)) {
                    *line = format!("{}={}", key, value);
                    found.insert(key);
                    break;
                }
            }
        }

        // Add missing keys
        for (key, value) in &updates {
            if !found.contains(key) {
                lines.push(format!("{}={}", key, value));
            }
        }

        std::fs::write(env_path, lines.join("\n") + "\n")
    }
}

/// Shared state for web server
pub type SharedConfig = Arc<RwLock<RuntimeConfig>>;

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
            description: "Total cost threshold in cents. 995 = 99.5¢ = 0.5% profit minimum",
            setting_type: "number",
            requires_restart: false,
            min: Some(900),
            max: Some(999),
        },
        SettingMeta {
            key: "dry_run",
            label: "Dry Run Mode",
            description: "Simulate trades without executing real orders",
            setting_type: "toggle",
            requires_restart: false,
            min: None,
            max: None,
        },
        SettingMeta {
            key: "priority_mode",
            label: "Priority Mode",
            description: "Enable priority queue for market scanning",
            setting_type: "toggle",
            requires_restart: true,
            min: None,
            max: None,
        },
        SettingMeta {
            key: "crypto_enabled",
            label: "Crypto Markets",
            description: "Enable BTC/ETH crypto market discovery",
            setting_type: "toggle",
            requires_restart: true,
            min: None,
            max: None,
        },
        SettingMeta {
            key: "min_liquidity_cents",
            label: "Min Liquidity (cents)",
            description: "Minimum trade size per side. 25000 = $250",
            setting_type: "number",
            requires_restart: false,
            min: Some(1000),
            max: Some(1000000),
        },
        SettingMeta {
            key: "max_liquidity_cents",
            label: "Max Liquidity (cents)",
            description: "Maximum trade size per side. 250000 = $2,500",
            setting_type: "number",
            requires_restart: false,
            min: Some(10000),
            max: Some(10000000),
        },
        SettingMeta {
            key: "max_daily_loss_cents",
            label: "Max Daily Loss (cents)",
            description: "Circuit breaker: halt trading if daily loss exceeds this",
            setting_type: "number",
            requires_restart: false,
            min: Some(10000),
            max: Some(10000000),
        },
        SettingMeta {
            key: "max_position_size_cents",
            label: "Max Position Size (cents)",
            description: "Maximum single position size allowed",
            setting_type: "number",
            requires_restart: false,
            min: Some(10000),
            max: Some(10000000),
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
        SettingMeta {
            key: "queue_sort_interval_secs",
            label: "Queue Sort Interval (sec)",
            description: "How often to re-sort the priority queue",
            setting_type: "number",
            requires_restart: false,
            min: Some(1),
            max: Some(60),
        },
        SettingMeta {
            key: "ws_reconnect_delay_secs",
            label: "WS Reconnect Delay (sec)",
            description: "Delay before reconnecting dropped WebSockets",
            setting_type: "number",
            requires_restart: false,
            min: Some(1),
            max: Some(30),
        },
        SettingMeta {
            key: "enabled_leagues",
            label: "Enabled Leagues",
            description: "Comma-separated list (empty = all). e.g., nba,nfl,epl",
            setting_type: "text",
            requires_restart: true,
            min: None,
            max: None,
        },
    ]
}

/// GET /api/config - Get current configuration
async fn get_config(State(config): State<SharedConfig>) -> impl IntoResponse {
    let cfg = config.read().await;
    Json(cfg.clone())
}

/// GET /api/meta - Get settings metadata
async fn get_meta() -> impl IntoResponse {
    Json(get_settings_meta())
}

/// POST /api/config - Update configuration
async fn update_config(
    State(config): State<SharedConfig>,
    Json(new_config): Json<RuntimeConfig>,
) -> impl IntoResponse {
    // Save to .env file
    if let Err(e) = new_config.save_to_env(".env") {
        warn!("[WEB] Failed to save config: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save config");
    }

    // Update in-memory config
    let mut cfg = config.write().await;
    *cfg = new_config;

    info!("[WEB] Config updated successfully");
    (StatusCode::OK, "Config saved")
}

/// POST /api/restart - Signal bot restart needed
async fn trigger_restart() -> impl IntoResponse {
    info!("[WEB] Restart requested via web UI");
    // In a real implementation, this would signal the main loop to restart
    // For now, we just acknowledge the request
    (StatusCode::OK, "Restart signal sent. Please restart the bot manually.")
}

/// GET / - Serve the HTML dashboard
async fn serve_dashboard() -> Html<&'static str> {
    Html(DASHBOARD_HTML)
}

/// Create and run the web server
pub async fn run_web_server(config: SharedConfig) {
    let port = std::env::var("WEB_CONFIG_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    let app = Router::new()
        .route("/", get(serve_dashboard))
        .route("/api/config", get(get_config).post(update_config))
        .route("/api/meta", get(get_meta))
        .route("/api/restart", post(trigger_restart))
        .with_state(config);

    let addr = format!("0.0.0.0:{}", port);
    info!("[WEB] Starting config dashboard on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    if let Err(e) = axum::serve(listener, app).await {
        warn!("[WEB] Server error: {}", e);
    }
}

/// Embedded HTML dashboard
const DASHBOARD_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Arb Bot Config</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #0d1117;
            color: #c9d1d9;
            padding: 20px;
            max-width: 800px;
            margin: 0 auto;
        }
        h1 {
            color: #58a6ff;
            margin-bottom: 8px;
            font-size: 24px;
        }
        .subtitle {
            color: #8b949e;
            margin-bottom: 24px;
            font-size: 14px;
        }
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
        .setting {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 12px 0;
            border-bottom: 1px solid #21262d;
        }
        .setting:last-child { border-bottom: none; }
        .setting-info {
            flex: 1;
        }
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
        .setting-control {
            min-width: 150px;
            text-align: right;
        }
        input[type="number"], input[type="text"] {
            background: #0d1117;
            border: 1px solid #30363d;
            border-radius: 6px;
            color: #c9d1d9;
            padding: 8px 12px;
            width: 120px;
            font-size: 14px;
        }
        input[type="number"]:focus, input[type="text"]:focus {
            border-color: #58a6ff;
            outline: none;
        }
        .toggle {
            position: relative;
            width: 50px;
            height: 26px;
        }
        .toggle input {
            opacity: 0;
            width: 0;
            height: 0;
        }
        .toggle-slider {
            position: absolute;
            cursor: pointer;
            top: 0; left: 0; right: 0; bottom: 0;
            background: #30363d;
            border-radius: 26px;
            transition: 0.3s;
        }
        .toggle-slider:before {
            position: absolute;
            content: "";
            height: 20px;
            width: 20px;
            left: 3px;
            bottom: 3px;
            background: white;
            border-radius: 50%;
            transition: 0.3s;
        }
        .toggle input:checked + .toggle-slider {
            background: #238636;
        }
        .toggle input:checked + .toggle-slider:before {
            transform: translateX(24px);
        }
        .btn {
            background: #238636;
            color: white;
            border: none;
            padding: 12px 24px;
            border-radius: 6px;
            font-size: 14px;
            font-weight: 600;
            cursor: pointer;
            margin-right: 8px;
        }
        .btn:hover { background: #2ea043; }
        .btn-secondary {
            background: #30363d;
        }
        .btn-secondary:hover { background: #3d444d; }
        .btn-danger {
            background: #da3633;
        }
        .btn-danger:hover { background: #f85149; }
        .actions {
            margin-top: 20px;
            display: flex;
            gap: 12px;
        }
        .status {
            padding: 12px;
            border-radius: 6px;
            margin-top: 16px;
            display: none;
        }
        .status.success {
            display: block;
            background: #238636;
            color: white;
        }
        .status.error {
            display: block;
            background: #da3633;
            color: white;
        }
        .status.warning {
            display: block;
            background: #9e6a03;
            color: white;
        }
        .loader {
            display: none;
            color: #8b949e;
            padding: 20px;
            text-align: center;
        }
        #content { display: none; }
    </style>
</head>
<body>
    <h1>Arbitrage Bot Configuration</h1>
    <p class="subtitle">Adjust settings in real-time. Settings with <span class="restart-badge">RESTART</span> require a bot restart to take effect.</p>

    <div class="loader" id="loader">Loading configuration...</div>
    <div id="status" class="status"></div>

    <div id="content">
        <div class="section">
            <div class="section-title">Trading Settings</div>
            <div id="trading-settings"></div>
        </div>

        <div class="section">
            <div class="section-title">Liquidity Limits</div>
            <div id="liquidity-settings"></div>
        </div>

        <div class="section">
            <div class="section-title">Circuit Breaker</div>
            <div id="circuit-settings"></div>
        </div>

        <div class="section">
            <div class="section-title">Timing & Performance</div>
            <div id="timing-settings"></div>
        </div>

        <div class="section">
            <div class="section-title">Market Selection</div>
            <div id="market-settings"></div>
        </div>

        <div class="actions">
            <button class="btn" onclick="saveConfig()">Save Changes</button>
            <button class="btn btn-secondary" onclick="loadConfig()">Reset</button>
            <button class="btn btn-danger" onclick="restartBot()">Restart Bot</button>
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
            'min_liquidity_cents': 'liquidity-settings',
            'max_liquidity_cents': 'liquidity-settings',
            'max_daily_loss_cents': 'circuit-settings',
            'max_position_size_cents': 'circuit-settings',
            'cooldown_secs': 'circuit-settings',
            'queue_sort_interval_secs': 'timing-settings',
            'ws_reconnect_delay_secs': 'timing-settings',
            'enabled_leagues': 'market-settings',
        };

        async function loadConfig() {
            document.getElementById('loader').style.display = 'block';
            document.getElementById('content').style.display = 'none';

            try {
                const [configRes, metaRes] = await Promise.all([
                    fetch('/api/config'),
                    fetch('/api/meta')
                ]);
                config = await configRes.json();
                meta = await metaRes.json();
                renderSettings();
                document.getElementById('content').style.display = 'block';
            } catch (e) {
                showStatus('Failed to load config: ' + e.message, 'error');
            }
            document.getElementById('loader').style.display = 'none';
        }

        function renderSettings() {
            // Clear sections
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
                    control = `
                        <label class="toggle">
                            <input type="checkbox" id="${setting.key}" ${value ? 'checked' : ''}>
                            <span class="toggle-slider"></span>
                        </label>
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
            });
        }

        async function saveConfig() {
            const newConfig = {};
            meta.forEach(setting => {
                const el = document.getElementById(setting.key);
                if (!el) return;

                if (setting.setting_type === 'toggle') {
                    newConfig[setting.key] = el.checked;
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
                    const needsRestart = meta.some(m => m.requires_restart && config[m.key] !== newConfig[m.key]);
                    if (needsRestart) {
                        showStatus('Config saved! Some changes require a bot restart.', 'warning');
                    } else {
                        showStatus('Config saved successfully!', 'success');
                    }
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
            const el = document.getElementById('status');
            el.textContent = msg;
            el.className = 'status ' + type;
            setTimeout(() => { el.className = 'status'; }, 5000);
        }

        loadConfig();
    </script>
</body>
</html>
"#;

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
