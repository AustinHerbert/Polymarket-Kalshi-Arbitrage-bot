#!/bin/bash

echo "=========================================="
echo "Starting Monitoring Dashboard"
echo "=========================================="
echo ""

# Check if monitor binary exists
if [ ! -f "target/release/monitor" ]; then
    echo "Monitor binary not found. Building..."
    cargo build --release --bin monitor
    echo ""
fi

# Kill existing monitor if running
screen -S monitor -X quit 2>/dev/null || true

# Start monitor in screen session
echo "Starting monitoring dashboard on port 3000..."
screen -dmS monitor bash -c "cd /root/Polymarket-Kalshi-Arbitrage-bot && ./target/release/monitor"

sleep 2

echo ""
echo "=========================================="
echo "✓ Monitoring Dashboard Started!"
echo "=========================================="
echo ""
echo "Access dashboard at: http://167.172.24.118:3000"
echo ""
echo "Commands:"
echo "  View monitor:    screen -r monitor"
echo "  Detach:          Ctrl+A, then D"
echo "  Stop monitor:    screen -S monitor -X quit"
echo ""
