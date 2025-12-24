#!/bin/bash
# Fetch real token IDs from Polymarket Gamma API
# Run this on the server to get token IDs for our target markets

set -e

echo "=== Fetching Token IDs from Polymarket Gamma API ==="
echo ""

# Bitcoin 2025 market
echo "1. Bitcoin 2025 Price Market"
echo "   Market slug: what-price-will-bitcoin-hit-in-2025"
echo ""
echo "   Fetching market data..."
curl -s "https://gamma-api.polymarket.com/markets?slug=what-price-will-bitcoin-hit-in-2025" > /tmp/btc_market.json

if [ -s /tmp/btc_market.json ]; then
    echo "   ✓ Found market data"
    echo ""
    echo "   Tokens (outcomes):"
    cat /tmp/btc_market.json | python3 -c "
import json, sys
data = json.load(sys.stdin)
if isinstance(data, list) and len(data) > 0:
    market = data[0]
    print(f'   Market: {market.get(\"question\", \"N/A\")}')
    print(f'   Tokens: {market.get(\"clob_token_ids\", [])}')
    print(f'   Outcomes: {market.get(\"outcomes\", [])}')
    for i, token_id in enumerate(market.get('clob_token_ids', [])):
        outcome = market.get('outcomes', [])[i] if i < len(market.get('outcomes', [])) else 'Unknown'
        print(f'   [{i}] {outcome}: {token_id}')
"
else
    echo "   ✗ No data returned"
fi

echo ""
echo "---"
echo ""

# Ethereum 2025 market
echo "2. Ethereum 2025 Price Market"
echo "   Market slug: what-price-will-ethereum-hit-in-2025"
echo ""
echo "   Fetching market data..."
curl -s "https://gamma-api.polymarket.com/markets?slug=what-price-will-ethereum-hit-in-2025" > /tmp/eth_market.json

if [ -s /tmp/eth_market.json ]; then
    echo "   ✓ Found market data"
    echo ""
    echo "   Tokens (outcomes):"
    cat /tmp/eth_market.json | python3 -c "
import json, sys
data = json.load(sys.stdin)
if isinstance(data, list) and len(data) > 0:
    market = data[0]
    print(f'   Market: {market.get(\"question\", \"N/A\")}')
    print(f'   Tokens: {market.get(\"clob_token_ids\", [])}')
    print(f'   Outcomes: {market.get(\"outcomes\", [])}')
    for i, token_id in enumerate(market.get('clob_token_ids', [])):
        outcome = market.get('outcomes', [])[i] if i < len(market.get('outcomes', [])) else 'Unknown'
        print(f'   [{i}] {outcome}: {token_id}')
"
else
    echo "   ✗ No data returned"
fi

echo ""
echo "=== Done ==="
echo ""
echo "To use these token IDs:"
echo "1. Choose which price outcomes you want to market make (e.g., BTC $110K-120K)"
echo "2. Update spread-farming-bot/src/config.rs with the token IDs"
echo "3. Test with: RUST_LOG=info cargo run --release"
