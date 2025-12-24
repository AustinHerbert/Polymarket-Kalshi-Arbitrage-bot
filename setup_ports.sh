#!/bin/bash

set -e

echo "=========================================="
echo "Setting up Bot Port Configuration"
echo "=========================================="
echo ""

# Check if .env exists in main directory
if [ ! -f ".env" ]; then
    echo "Error: .env file not found in main directory!"
    echo "Please copy .env.example to .env first:"
    echo "  cp .env.example .env"
    exit 1
fi

# Check if original-bot directory exists
if [ ! -d "original-bot" ]; then
    echo "Error: original-bot directory not found!"
    echo "Please run ./setup_fork.sh first"
    exit 1
fi

# Add or update WEB_CONFIG_PORT in main .env
echo "Configuring main bot to use port 8080..."
if grep -q "^WEB_CONFIG_PORT=" .env; then
    # Port already set, update it
    sed -i 's/^WEB_CONFIG_PORT=.*/WEB_CONFIG_PORT=8080/' .env
    echo "  ✓ Updated WEB_CONFIG_PORT=8080 in .env"
else
    # Add port setting
    echo "" >> .env
    echo "# Web dashboard port (default: 8080)" >> .env
    echo "WEB_CONFIG_PORT=8080" >> .env
    echo "  ✓ Added WEB_CONFIG_PORT=8080 to .env"
fi

# Configure original-bot .env
echo "Configuring original bot to use port 8081..."
if [ -f "original-bot/.env" ]; then
    if grep -q "^WEB_CONFIG_PORT=" original-bot/.env; then
        sed -i 's/^WEB_CONFIG_PORT=.*/WEB_CONFIG_PORT=8081/' original-bot/.env
        echo "  ✓ Updated WEB_CONFIG_PORT=8081 in original-bot/.env"
    else
        echo "" >> original-bot/.env
        echo "# Web dashboard port (default: 8080)" >> original-bot/.env
        echo "WEB_CONFIG_PORT=8081" >> original-bot/.env
        echo "  ✓ Added WEB_CONFIG_PORT=8081 to original-bot/.env"
    fi
else
    echo "  ! Warning: original-bot/.env not found, will be created on first run"
fi

echo ""
echo "=========================================="
echo "✓ Port Configuration Complete!"
echo "=========================================="
echo ""
echo "Port Assignments:"
echo "  - My Bot Dashboard:       http://167.172.24.118:8080"
echo "  - Original Bot Dashboard: http://167.172.24.118:8081"
echo "  - Monitoring Dashboard:   http://167.172.24.118:3000"
echo ""
echo "Next steps:"
echo "  1. Build monitor: cargo build --release --bin monitor"
echo "  2. Start monitor: ./start_monitor.sh"
echo "  3. Start bots:    ./run_both_bots.sh"
echo ""
