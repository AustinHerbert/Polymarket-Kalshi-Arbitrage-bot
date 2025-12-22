#!/bin/bash

# ============================================================
# EASY SETUP SCRIPT FOR ARBITRAGE BOT
# ============================================================
# This script helps you set up the bot step by step
# ============================================================

echo ""
echo "=============================================="
echo "   ARBITRAGE BOT - EASY SETUP"
echo "=============================================="
echo ""

# Check if .env already exists
if [ -f ".env" ]; then
    echo "[!] A .env file already exists."
    read -p "Do you want to overwrite it? (y/n): " overwrite
    if [ "$overwrite" != "y" ]; then
        echo "Setup cancelled. Your existing .env was kept."
        exit 0
    fi
fi

# Copy the example file
cp .env.example .env
echo "[OK] Created .env file from template"
echo ""

# Collect Kalshi credentials
echo "=============================================="
echo "STEP 1: KALSHI API CREDENTIALS"
echo "=============================================="
echo "Get these from: https://kalshi.com/settings/api"
echo ""

read -p "Enter your Kalshi API Key ID: " kalshi_key
read -p "Enter the FULL PATH to your Kalshi private key file: " kalshi_path

# Collect Polymarket credentials
echo ""
echo "=============================================="
echo "STEP 2: POLYMARKET CREDENTIALS"
echo "=============================================="
echo "Get these from your Polymarket wallet"
echo ""

read -p "Enter your Polymarket private key (starts with 0x): " poly_key
read -p "Enter your Polymarket wallet address (starts with 0x): " poly_funder

# Ask about priority mode
echo ""
echo "=============================================="
echo "STEP 3: MODE SELECTION"
echo "=============================================="
echo ""
read -p "Enable Priority Mode (new features)? (y/n): " priority

if [ "$priority" = "y" ]; then
    priority_mode=1
    echo "[OK] Priority Mode will be ENABLED"
else
    priority_mode=0
    echo "[OK] Priority Mode will be DISABLED (using standard mode)"
fi

# Update the .env file
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS uses different sed syntax
    sed -i '' "s|KALSHI_API_KEY_ID=.*|KALSHI_API_KEY_ID=$kalshi_key|" .env
    sed -i '' "s|KALSHI_PRIVATE_KEY_PATH=.*|KALSHI_PRIVATE_KEY_PATH=$kalshi_path|" .env
    sed -i '' "s|POLY_PRIVATE_KEY=.*|POLY_PRIVATE_KEY=$poly_key|" .env
    sed -i '' "s|POLY_FUNDER=.*|POLY_FUNDER=$poly_funder|" .env
    sed -i '' "s|PRIORITY_MODE=.*|PRIORITY_MODE=$priority_mode|" .env
else
    # Linux
    sed -i "s|KALSHI_API_KEY_ID=.*|KALSHI_API_KEY_ID=$kalshi_key|" .env
    sed -i "s|KALSHI_PRIVATE_KEY_PATH=.*|KALSHI_PRIVATE_KEY_PATH=$kalshi_path|" .env
    sed -i "s|POLY_PRIVATE_KEY=.*|POLY_PRIVATE_KEY=$poly_key|" .env
    sed -i "s|POLY_FUNDER=.*|POLY_FUNDER=$poly_funder|" .env
    sed -i "s|PRIORITY_MODE=.*|PRIORITY_MODE=$priority_mode|" .env
fi

echo ""
echo "=============================================="
echo "   SETUP COMPLETE!"
echo "=============================================="
echo ""
echo "Your .env file has been configured."
echo ""
echo "IMPORTANT: The bot is set to DRY_RUN=1 (simulation mode)"
echo "This means it will NOT make real trades yet."
echo ""
echo "To start the bot in TEST MODE (recommended first):"
echo "   cargo run --release"
echo ""
echo "Once you're confident, edit .env and change DRY_RUN=0"
echo "to enable real trading."
echo ""
echo "=============================================="
