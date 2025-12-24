#!/bin/bash

# This script updates run_both_bots.sh to properly set WEB_CONFIG_PORT

cat > run_both_bots.sh << 'EOF'
#!/bin/bash

set -e

echo "=========================================="
echo "Running Both Bots for Comparison"
echo "=========================================="
echo ""

# Check if original-bot directory exists
if [ ! -d "original-bot" ]; then
    echo "Error: original-bot directory not found!"
    echo "Please run ./setup_fork.sh first"
    exit 1
fi

# Create logs directory
mkdir -p logs

# Get current timestamp for log files
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

echo "Starting bots in screen sessions..."
echo ""

# Kill any existing sessions
screen -S my-bot -X quit 2>/dev/null || true
screen -S original-bot -X quit 2>/dev/null || true

# Start your modified bot (port 8080)
echo "Starting YOUR bot in screen session 'my-bot' (port 8080)..."
screen -dmS my-bot bash -c "cd /root/Polymarket-Kalshi-Arbitrage-bot && export WEB_CONFIG_PORT=8080 && RUST_LOG=info cargo run --release 2>&1 | tee logs/my-bot_${TIMESTAMP}.log"

# Start original bot (port 8081)
echo "Starting ORIGINAL bot in screen session 'original-bot' (port 8081)..."
screen -dmS original-bot bash -c "cd /root/Polymarket-Kalshi-Arbitrage-bot/original-bot && export WEB_CONFIG_PORT=8081 && RUST_LOG=info cargo run --release 2>&1 | tee ../logs/original-bot_${TIMESTAMP}.log"

sleep 2

echo ""
echo "=========================================="
echo "✓ Both Bots Started!"
echo "=========================================="
echo ""
echo "Screen sessions:"
echo "  - my-bot        (your modified version)"
echo "  - original-bot  (original version)"
echo ""
echo "Web Dashboards:"
echo "  - My Bot:        http://167.172.24.118:8080"
echo "  - Original Bot:  http://167.172.24.118:8081"
echo "  - Monitor:       http://167.172.24.118:3000"
echo ""
echo "Log files:"
echo "  - logs/my-bot_${TIMESTAMP}.log"
echo "  - logs/original-bot_${TIMESTAMP}.log"
echo ""
echo "Commands:"
echo "  View your bot:      screen -r my-bot"
echo "  View original bot:  screen -r original-bot"
echo "  Detach from screen: Ctrl+A, then D"
echo "  Stop both bots:     ./stop_bots.sh"
echo "  Compare results:    ./compare_bots.sh"
echo ""
echo "Position tracking files:"
echo "  - positions.json (your bot)"
echo "  - original-bot/positions.json (original bot)"
echo ""
echo "=========================================="
echo ""
echo "Both bots are running in DRY_RUN mode (no real trades)"
echo "Let them run for 24 hours, then use ./compare_bots.sh"
echo ""
EOF

chmod +x run_both_bots.sh

echo "✓ Updated run_both_bots.sh with port configuration"
echo ""
echo "Port assignments:"
echo "  - My Bot:        Port 8080"
echo "  - Original Bot:  Port 8081"
echo "  - Monitor:       Port 3000"
