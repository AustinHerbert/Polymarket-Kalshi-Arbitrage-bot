//! Server Monitoring Dashboard
//!
//! Provides a web-based dashboard on port 3000 to monitor:
//! - Running bot processes
//! - Screen sessions
//! - Performance metrics
//! - System resources
//! - Recent logs

use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::process::Command;

const MONITOR_PORT: u16 = 3000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotStatus {
    pub name: String,
    pub pid: Option<u32>,
    pub status: String,
    pub memory_mb: Option<f64>,
    pub cpu_percent: Option<f64>,
    pub uptime: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenSession {
    pub id: String,
    pub name: String,
    pub attached: bool,
    pub created: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorData {
    pub bots: Vec<BotStatus>,
    pub screens: Vec<ScreenSession>,
    pub system_load: String,
    pub memory_usage: String,
    pub disk_usage: String,
    pub timestamp: String,
}

/// Start the monitoring dashboard server
pub async fn run_monitor_server() {
    let app = Router::new()
        .route("/", get(serve_dashboard))
        .route("/api/status", get(get_status));

    let addr = format!("0.0.0.0:{}", MONITOR_PORT);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind monitoring dashboard");

    println!("🖥️  Monitoring Dashboard: http://0.0.0.0:{}", MONITOR_PORT);

    axum::serve(listener, app)
        .await
        .expect("Failed to start monitoring server");
}

async fn serve_dashboard() -> Html<String> {
    Html(DASHBOARD_HTML.to_string())
}

async fn get_status() -> impl IntoResponse {
    let data = collect_system_data().await;
    Json(data)
}

async fn collect_system_data() -> MonitorData {
    let bots = get_running_bots();
    let screens = get_screen_sessions();
    let (load, memory, disk) = get_system_stats();

    MonitorData {
        bots,
        screens,
        system_load: load,
        memory_usage: memory,
        disk_usage: disk,
        timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    }
}

fn get_running_bots() -> Vec<BotStatus> {
    let mut bots = Vec::new();

    // Find all prediction-market-arbitrage processes
    let output = Command::new("ps")
        .args(&["aux"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("prediction-market-arbitrage") && !line.contains("grep") {
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() >= 11 {
                    let pid = fields[1].parse::<u32>().ok();
                    let cpu = fields[2].parse::<f64>().ok();
                    let mem = fields[3].parse::<f64>().ok();

                    // Determine bot name from process or default
                    let name = if line.contains("original-bot") {
                        "Original Bot".to_string()
                    } else if line.contains("my-bot") {
                        "My Bot".to_string()
                    } else {
                        "Arbitrage Bot".to_string()
                    };

                    bots.push(BotStatus {
                        name,
                        pid,
                        status: "Running".to_string(),
                        memory_mb: mem.map(|m| m * 10.0), // Rough estimate
                        cpu_percent: cpu,
                        uptime: Some(fields[9].to_string()),
                    });
                }
            }
        }
    }

    // If no bots found, add placeholder
    if bots.is_empty() {
        bots.push(BotStatus {
            name: "No bots running".to_string(),
            pid: None,
            status: "Stopped".to_string(),
            memory_mb: None,
            cpu_percent: None,
            uptime: None,
        });
    }

    bots
}

fn get_screen_sessions() -> Vec<ScreenSession> {
    let mut sessions = Vec::new();

    let output = Command::new("screen")
        .args(&["-ls"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("Detached") || line.contains("Attached") {
                let parts: Vec<&str> = line.trim().split_whitespace().collect();
                if parts.len() >= 3 {
                    let id_name: Vec<&str> = parts[0].split('.').collect();
                    let attached = line.contains("Attached");

                    sessions.push(ScreenSession {
                        id: id_name.get(0).unwrap_or(&"").to_string(),
                        name: id_name.get(1).unwrap_or(&"unknown").to_string(),
                        attached,
                        created: parts.get(1).unwrap_or(&"").to_string(),
                    });
                }
            }
        }
    }

    sessions
}

fn get_system_stats() -> (String, String, String) {
    let load = Command::new("uptime")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_else(|_| "N/A".to_string());

    let memory = Command::new("free")
        .args(&["-h"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().nth(1).unwrap_or("N/A").to_string())
        .unwrap_or_else(|_| "N/A".to_string());

    let disk = Command::new("df")
        .args(&["-h", "/"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().nth(1).unwrap_or("N/A").to_string())
        .unwrap_or_else(|_| "N/A".to_string());

    (load, memory, disk)
}

const DASHBOARD_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Bot Monitor - Server Dashboard</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 20px;
        }

        .container {
            max-width: 1400px;
            margin: 0 auto;
        }

        .header {
            text-align: center;
            color: white;
            margin-bottom: 30px;
        }

        .header h1 {
            font-size: 2.5rem;
            margin-bottom: 10px;
            text-shadow: 2px 2px 4px rgba(0,0,0,0.2);
        }

        .header .subtitle {
            font-size: 1.1rem;
            opacity: 0.9;
        }

        .server-info {
            background: white;
            border-radius: 12px;
            padding: 20px;
            margin-bottom: 20px;
            box-shadow: 0 4px 6px rgba(0,0,0,0.1);
        }

        .server-info h2 {
            color: #667eea;
            margin-bottom: 15px;
            font-size: 1.3rem;
        }

        .grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
            gap: 20px;
            margin-bottom: 20px;
        }

        .card {
            background: white;
            border-radius: 12px;
            padding: 20px;
            box-shadow: 0 4px 6px rgba(0,0,0,0.1);
        }

        .card h2 {
            color: #667eea;
            margin-bottom: 15px;
            font-size: 1.3rem;
            border-bottom: 2px solid #667eea;
            padding-bottom: 10px;
        }

        .bot-item {
            background: #f8f9fa;
            border-left: 4px solid #28a745;
            padding: 15px;
            margin-bottom: 10px;
            border-radius: 6px;
        }

        .bot-item.stopped {
            border-left-color: #dc3545;
        }

        .bot-name {
            font-weight: 600;
            font-size: 1.1rem;
            color: #333;
            margin-bottom: 8px;
        }

        .bot-details {
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            gap: 8px;
            font-size: 0.9rem;
            color: #666;
        }

        .detail-item {
            display: flex;
            align-items: center;
        }

        .detail-label {
            font-weight: 500;
            margin-right: 5px;
        }

        .status-badge {
            display: inline-block;
            padding: 4px 12px;
            border-radius: 12px;
            font-size: 0.85rem;
            font-weight: 600;
        }

        .status-running {
            background: #d4edda;
            color: #155724;
        }

        .status-stopped {
            background: #f8d7da;
            color: #721c24;
        }

        .screen-item {
            background: #f8f9fa;
            border-left: 4px solid #007bff;
            padding: 12px;
            margin-bottom: 8px;
            border-radius: 6px;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }

        .screen-item.attached {
            border-left-color: #28a745;
        }

        .screen-name {
            font-weight: 600;
            color: #333;
        }

        .screen-id {
            color: #666;
            font-size: 0.9rem;
        }

        .stats-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 15px;
            margin-top: 15px;
        }

        .stat-box {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 15px;
            border-radius: 8px;
            text-align: center;
        }

        .stat-label {
            font-size: 0.85rem;
            opacity: 0.9;
            margin-bottom: 5px;
        }

        .stat-value {
            font-size: 1.5rem;
            font-weight: 700;
        }

        .refresh-notice {
            text-align: center;
            color: white;
            margin-top: 20px;
            font-size: 0.9rem;
            opacity: 0.8;
        }

        .quick-links {
            background: white;
            border-radius: 12px;
            padding: 20px;
            margin-top: 20px;
            box-shadow: 0 4px 6px rgba(0,0,0,0.1);
        }

        .quick-links h2 {
            color: #667eea;
            margin-bottom: 15px;
        }

        .link-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 10px;
        }

        .quick-link {
            display: block;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 12px 20px;
            border-radius: 6px;
            text-decoration: none;
            text-align: center;
            font-weight: 600;
            transition: transform 0.2s;
        }

        .quick-link:hover {
            transform: translateY(-2px);
            box-shadow: 0 4px 8px rgba(0,0,0,0.2);
        }

        .timestamp {
            text-align: center;
            color: white;
            margin-top: 10px;
            font-size: 0.9rem;
        }

        @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.5; }
        }

        .loading {
            animation: pulse 2s infinite;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🖥️ Arbitrage Bot Monitor</h1>
            <div class="subtitle">Real-time server and process monitoring</div>
        </div>

        <div class="server-info">
            <h2>📡 Server: 167.172.24.118</h2>
            <div class="stats-grid" id="systemStats">
                <div class="stat-box loading">
                    <div class="stat-label">System Load</div>
                    <div class="stat-value">Loading...</div>
                </div>
                <div class="stat-box loading">
                    <div class="stat-label">Memory</div>
                    <div class="stat-value">Loading...</div>
                </div>
                <div class="stat-box loading">
                    <div class="stat-label">Disk Usage</div>
                    <div class="stat-value">Loading...</div>
                </div>
            </div>
        </div>

        <div class="grid">
            <div class="card">
                <h2>🤖 Running Bots</h2>
                <div id="botsList">
                    <div class="bot-item loading">
                        <div class="bot-name">Loading bot information...</div>
                    </div>
                </div>
            </div>

            <div class="card">
                <h2>📺 Screen Sessions</h2>
                <div id="screensList">
                    <div class="screen-item loading">
                        <div class="screen-name">Loading sessions...</div>
                    </div>
                </div>
            </div>
        </div>

        <div class="quick-links">
            <h2>🔗 Quick Access</h2>
            <div class="link-grid">
                <a href="http://167.172.24.118:8080" class="quick-link" target="_blank">My Bot Dashboard (8080)</a>
                <a href="http://167.172.24.118:8081" class="quick-link" target="_blank">Original Bot Dashboard (8081)</a>
            </div>
        </div>

        <div class="timestamp" id="timestamp">Last updated: --</div>
        <div class="refresh-notice">Auto-refreshing every 5 seconds</div>
    </div>

    <script>
        async function updateDashboard() {
            try {
                const response = await fetch('/api/status');
                const data = await response.json();

                // Update bots list
                const botsList = document.getElementById('botsList');
                if (data.bots && data.bots.length > 0) {
                    botsList.innerHTML = data.bots.map(bot => {
                        const isStopped = bot.status === 'Stopped' || !bot.pid;
                        return `
                            <div class="bot-item ${isStopped ? 'stopped' : ''}">
                                <div class="bot-name">
                                    ${bot.name}
                                    <span class="status-badge ${isStopped ? 'status-stopped' : 'status-running'}">
                                        ${bot.status}
                                    </span>
                                </div>
                                ${bot.pid ? `
                                <div class="bot-details">
                                    <div class="detail-item"><span class="detail-label">PID:</span> ${bot.pid}</div>
                                    <div class="detail-item"><span class="detail-label">CPU:</span> ${bot.cpu_percent?.toFixed(1) || 'N/A'}%</div>
                                    <div class="detail-item"><span class="detail-label">Memory:</span> ${bot.memory_mb?.toFixed(0) || 'N/A'} MB</div>
                                    <div class="detail-item"><span class="detail-label">Uptime:</span> ${bot.uptime || 'N/A'}</div>
                                </div>
                                ` : ''}
                            </div>
                        `;
                    }).join('');
                } else {
                    botsList.innerHTML = '<div class="bot-item stopped"><div class="bot-name">No bots running</div></div>';
                }

                // Update screen sessions
                const screensList = document.getElementById('screensList');
                if (data.screens && data.screens.length > 0) {
                    screensList.innerHTML = data.screens.map(screen => `
                        <div class="screen-item ${screen.attached ? 'attached' : ''}">
                            <div>
                                <div class="screen-name">${screen.name}</div>
                                <div class="screen-id">ID: ${screen.id} • ${screen.created}</div>
                            </div>
                            <span class="status-badge ${screen.attached ? 'status-running' : 'status-stopped'}">
                                ${screen.attached ? 'Attached' : 'Detached'}
                            </span>
                        </div>
                    `).join('');
                } else {
                    screensList.innerHTML = '<div class="screen-item"><div class="screen-name">No screen sessions</div></div>';
                }

                // Update system stats
                const loadMatch = data.system_load.match(/load average: ([\d.]+)/);
                const memMatch = data.memory_usage.match(/(\d+)Gi.*?(\d+)Gi/);
                const diskMatch = data.disk_usage.match(/(\d+)%/);

                document.getElementById('systemStats').innerHTML = `
                    <div class="stat-box">
                        <div class="stat-label">System Load</div>
                        <div class="stat-value">${loadMatch ? loadMatch[1] : 'N/A'}</div>
                    </div>
                    <div class="stat-box">
                        <div class="stat-label">Memory Used</div>
                        <div class="stat-value">${memMatch ? `${memMatch[1]}/${memMatch[2]}G` : 'N/A'}</div>
                    </div>
                    <div class="stat-box">
                        <div class="stat-label">Disk Usage</div>
                        <div class="stat-value">${diskMatch ? diskMatch[1] + '%' : 'N/A'}</div>
                    </div>
                `;

                // Update timestamp
                document.getElementById('timestamp').textContent = `Last updated: ${data.timestamp}`;

            } catch (error) {
                console.error('Failed to update dashboard:', error);
            }
        }

        // Initial load
        updateDashboard();

        // Auto-refresh every 5 seconds
        setInterval(updateDashboard, 5000);
    </script>
</body>
</html>
"#;
