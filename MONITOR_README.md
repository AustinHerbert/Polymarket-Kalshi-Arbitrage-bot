# 🖥️ Bot Monitoring Dashboard

This guide shows you how to set up and use the web-based monitoring dashboard to track all your running bots from your browser.

## 🎯 What You Get

### 3 Web Dashboards

1. **Main Monitoring Dashboard (Port 3000)**
   - See all running bots at a glance
   - View system resources (CPU, memory, disk)
   - Check screen sessions
   - Quick links to bot dashboards
   - Auto-refreshes every 5 seconds
   - **Access:** http://167.172.24.118:3000

2. **My Bot Dashboard (Port 8080)**
   - Your modified bot's web interface
   - Configuration settings
   - Real-time status and analytics
   - Trade history
   - **Access:** http://167.172.24.118:8080

3. **Original Bot Dashboard (Port 8081)**
   - Original bot's web interface
   - Same features as above
   - For comparison testing
   - **Access:** http://167.172.24.118:8081

## 🚀 Quick Start

### Step 1: One-Time Setup

Run these commands on your server:

```bash
cd ~/Polymarket-Kalshi-Arbitrage-bot

# 1. Configure port assignments
./setup_ports.sh

# 2. Update run script with new ports
./update_run_both_bots.sh

# 3. Build the monitor
cargo build --release --bin monitor
```

### Step 2: Start Everything

```bash
# Stop any old processes
./stop_all.sh

# Start the monitoring dashboard
./start_monitor.sh

# Start both comparison bots
./run_both_bots.sh
```

### Step 3: Access Dashboards

Open in your web browser:

- **Monitoring Dashboard:** http://167.172.24.118:3000
- **My Bot:** http://167.172.24.118:8080
- **Original Bot:** http://167.172.24.118:8081

## 📊 Monitoring Dashboard Features

The main monitoring dashboard (port 3000) shows:

### System Overview
- ⚡ System load average
- 💾 Memory usage
- 💿 Disk usage
- 📅 Last update timestamp

### Running Bots
- Bot names and status
- Process IDs (PIDs)
- CPU usage per bot
- Memory consumption
- Uptime

### Screen Sessions
- All active screen sessions
- Session IDs and names
- Attached/Detached status
- Creation timestamps

### Quick Access
- Direct links to both bot dashboards
- Auto-refresh every 5 seconds
- Mobile-responsive design

## 🛠️ Common Commands

### Start Services
```bash
./start_monitor.sh          # Start monitoring dashboard
./run_both_bots.sh          # Start both bots
```

### Stop Services
```bash
./stop_all.sh               # Stop everything
screen -S monitor -X quit   # Stop just monitor
screen -S my-bot -X quit    # Stop just my-bot
screen -S original-bot -X quit  # Stop just original-bot
```

### View Logs
```bash
screen -r monitor           # View monitor output
screen -r my-bot            # View my-bot output
screen -r original-bot      # View original-bot output

# Detach from screen: Press Ctrl+A, then D
```

### Check Status
```bash
# List all screen sessions
screen -ls

# Check running processes
ps aux | grep -E 'prediction-market|monitor'

# View recent bot logs
tail -f logs/my-bot_*.log
tail -f logs/original-bot_*.log
```

## 🔧 Troubleshooting

### Monitor won't start
```bash
# Check if port 3000 is already in use
ss -tlnp | grep 3000

# Kill process using port
pkill -9 monitor

# Rebuild and restart
cargo build --release --bin monitor
./start_monitor.sh
```

### Bot dashboard won't load
```bash
# Check if bot is actually running
ps aux | grep prediction-market-arbitrage

# Check which ports are in use
ss -tlnp | grep -E '8080|8081'

# Restart bots
./stop_all.sh
./run_both_bots.sh
```

### Port conflicts
If you get "Address already in use" errors:

```bash
# Stop everything
./stop_all.sh

# Kill any remaining processes
pkill -9 prediction-market-arbitrage
pkill -9 monitor

# Start fresh
./start_monitor.sh
./run_both_bots.sh
```

### Can't access from browser
```bash
# Check firewall allows incoming connections
sudo ufw status
sudo ufw allow 3000/tcp
sudo ufw allow 8080/tcp
sudo ufw allow 8081/tcp

# Verify services are listening
ss -tlnp | grep -E '3000|8080|8081'
```

## 📁 File Structure

```
Polymarket-Kalshi-Arbitrage-bot/
├── src/
│   ├── bin/
│   │   └── monitor.rs          # Monitoring dashboard binary
│   └── monitor_dashboard.rs     # Dashboard implementation
├── setup_ports.sh               # Configure ports (one-time)
├── update_run_both_bots.sh      # Update run script (one-time)
├── start_monitor.sh             # Start monitoring dashboard
├── run_both_bots.sh             # Start both comparison bots
├── stop_all.sh                  # Stop everything
└── logs/                        # Bot logs
```

## 🎨 Dashboard Features

### Main Monitor (3000)
- Real-time process monitoring
- System resource tracking
- Screen session management
- Quick navigation to bot dashboards
- Auto-refresh every 5 seconds
- Clean, professional dark theme
- Mobile responsive

### Bot Dashboards (8080, 8081)
Each bot has its own dashboard with:
- Status bar (DRY RUN/LIVE mode)
- Real-time metrics (uptime, profit, trades)
- Performance analytics tab
- Trade history tab
- Configuration editor
- Auto-refresh

## 💡 Pro Tips

1. **Bookmark the monitor:** Add http://167.172.24.118:3000 to your browser bookmarks

2. **Use split screen:** View monitor + both bot dashboards side-by-side

3. **Check logs:** If a bot shows as running but dashboard won't load, check the logs in `screen -r <bot-name>`

4. **Performance:** The monitor is lightweight and won't impact bot performance

5. **Remote access:** Access from any device on your network using the IP address

## 🔒 Security Notes

- Dashboards are exposed on 0.0.0.0 (all interfaces)
- Use firewall rules to restrict access if needed
- No authentication is built-in
- Consider using SSH tunneling for production:
  ```bash
  ssh -L 3000:localhost:3000 root@167.172.24.118
  # Then access via http://localhost:3000
  ```

## 📝 Next Steps

After everything is running:

1. Open http://167.172.24.118:3000 in your browser
2. Verify both bots show as "Running"
3. Click the quick links to check each bot dashboard
4. Let them run for 24 hours
5. Use `./compare_bots.sh` to see which performed better

Enjoy your new monitoring setup! 🎉
