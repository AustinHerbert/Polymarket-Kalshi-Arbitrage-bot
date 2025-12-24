#!/bin/bash
# Quick Start Script for Spread Farming Bot

set -e

echo "🌾 Spread Farming Bot - Quick Start"
echo "===================================="
echo ""

# Check if .env exists
if [ ! -f .env ]; then
    echo "❌ .env file not found!"
    echo "   Please create .env from .env.example:"
    echo "   cp .env.example .env"
    echo "   nano .env  # Add your credentials"
    exit 1
fi

# Check if credentials are set
if grep -q "YOUR_WALLET_PRIVATE_KEY_HERE" .env; then
    echo "⚠️  WARNING: Credentials not set in .env"
    echo "   Please edit .env and add:"
    echo "   - POLY_PRIVATE_KEY (your wallet private key)"
    echo "   - POLY_FUNDER (your wallet address)"
    echo ""
    echo "   Using same wallet as arbitrage bot is fine."
    echo ""
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Check if binary exists
if [ ! -f target/release/spread-farming-bot ]; then
    echo "📦 Building spread farming bot..."
    cargo build --release
fi

echo ""
echo "✅ Starting spread farming bot in DRY RUN mode..."
echo "   (No real orders will be placed)"
echo ""
echo "Press Ctrl+C to stop"
echo ""

# Run the bot
./target/release/spread-farming-bot
