#!/bin/bash

# ============================================================
# RUN BOT SCRIPT
# ============================================================
# Simple script to run the arbitrage bot
# ============================================================

echo ""
echo "=============================================="
echo "   STARTING ARBITRAGE BOT"
echo "=============================================="

# Check if .env exists
if [ ! -f ".env" ]; then
    echo "[ERROR] No .env file found!"
    echo ""
    echo "Run the setup first:"
    echo "   ./scripts/easy_setup.sh"
    echo ""
    exit 1
fi

# Check mode
if grep -q "DRY_RUN=1" .env; then
    echo ""
    echo "[SAFE MODE] DRY_RUN is ON - No real trades will be made"
    echo ""
else
    echo ""
    echo "[!] WARNING: DRY_RUN is OFF - Real trades WILL be made!"
    echo ""
    read -p "Are you sure you want to continue? (yes/no): " confirm
    if [ "$confirm" != "yes" ]; then
        echo "Cancelled."
        exit 0
    fi
fi

if grep -q "PRIORITY_MODE=1" .env; then
    echo "[MODE] Priority Mode is ENABLED"
else
    echo "[MODE] Standard Mode (Priority Mode disabled)"
fi

echo ""
echo "Starting bot... (Press Ctrl+C to stop)"
echo "=============================================="
echo ""

# Run the bot
cargo run --release
