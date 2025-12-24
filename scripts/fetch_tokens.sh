#!/bin/bash
# Fetch Polymarket token IDs using curl and jq
# Run this on the server: bash fetch_tokens.sh

set -e

GAMMA_API="https://gamma-api.polymarket.com"

echo "==================================================="
echo "Polymarket Token ID Fetcher"
echo "==================================================="
echo ""

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo "Installing jq..."
    apt-get update && apt-get install -y jq
fi

# Function to fetch and display events
fetch_events() {
    local search="$1"
    local limit="${2:-20}"

    echo "Fetching active events (searching for: ${search:-all})..."

    # Fetch events sorted by volume
    RESPONSE=$(curl -s "${GAMMA_API}/events?closed=false&limit=${limit}&order=volume24hr&ascending=false")

    if [ -z "$RESPONSE" ] || [ "$RESPONSE" = "[]" ]; then
        echo "No events found or API error"
        return 1
    fi

    # If search term provided, filter
    if [ -n "$search" ]; then
        RESPONSE=$(echo "$RESPONSE" | jq -r "[.[] | select(.title | ascii_downcase | contains(\"$search\"))]")
    fi

    echo ""
    echo "$RESPONSE" | jq -r '.[] | "
==================================================
EVENT: \(.title)
Slug: \(.slug)
Volume 24h: $\(.volume24hr // 0)
Total Volume: $\(.volume // 0)
Liquidity: $\(.liquidity // 0)
Active: \(.active) | Closed: \(.closed)
--------------------------------------------------"'

    # Extract markets with token IDs
    echo ""
    echo "==================================================="
    echo "MARKETS WITH TOKEN IDS"
    echo "==================================================="

    echo "$RESPONSE" | jq -r '.[] | .markets[]? | select(.clobTokenIds != null) | "
Market: \(.question // .slug)
Slug: \(.slug)
Condition ID: \(.conditionId)
Token IDs (clobTokenIds): \(.clobTokenIds)
Active: \(.active) | Closed: \(.closed) | OrderBook: \(.enableOrderBook)
Outcomes: \(.outcomes)
Prices: \(.outcomePrices)
--------------------------------------------------"'
}

# Function to fetch specific slug
fetch_slug() {
    local slug="$1"
    echo "Fetching event with slug: $slug"

    # Try as event
    RESPONSE=$(curl -s "${GAMMA_API}/events?slug=${slug}")

    if [ "$RESPONSE" = "[]" ] || [ -z "$RESPONSE" ]; then
        # Try as market
        RESPONSE=$(curl -s "${GAMMA_API}/markets?slug=${slug}")
    fi

    if [ "$RESPONSE" = "[]" ] || [ -z "$RESPONSE" ]; then
        echo "No market found for slug: $slug"
        return 1
    fi

    echo "$RESPONSE" | jq '.'
}

# Function to get top volume markets
fetch_top_markets() {
    echo "Fetching top 50 active markets by volume..."

    curl -s "${GAMMA_API}/events?closed=false&limit=50&order=volume24hr&ascending=false" | \
    jq -r '.[] | .markets[]? | select(.clobTokenIds != null and .enableOrderBook == true and .active == true) | {
        slug: .slug,
        question: (.question // .slug)[:60],
        volume24h: (.volume24hr // 0),
        liquidity: (.liquidity // 0),
        conditionId: .conditionId,
        clobTokenIds: .clobTokenIds,
        outcomes: .outcomes,
        outcomePrices: .outcomePrices
    }' | jq -s 'sort_by(-.volume24h) | .[:20]'
}

# Function to output Rust config format
output_rust_config() {
    local json="$1"

    echo ""
    echo "==================================================="
    echo "RUST CONFIG FORMAT (copy to config.rs)"
    echo "==================================================="
    echo ""

    echo "$json" | jq -r '.[] | select(.clobTokenIds != null) |
    "// Market: \(.question // .slug)
// Volume 24h: $\(.volume24h)
MarketConfig {
    name: \"\(.slug)\",
    yes_token: \"\((.clobTokenIds | fromjson)[0])\",
    no_token: \"\((.clobTokenIds | fromjson)[1])\",
    min_spread: 0.02,
    order_size: 10.0,
},
"'
}

# Main
case "${1:-}" in
    "bitcoin")
        fetch_events "bitcoin"
        ;;
    "ethereum")
        fetch_events "ethereum"
        ;;
    "crypto")
        fetch_events "bitcoin" 10
        echo ""
        fetch_events "ethereum" 10
        ;;
    "top")
        MARKETS=$(fetch_top_markets)
        echo "$MARKETS" | jq '.'
        output_rust_config "$MARKETS"
        ;;
    "slug")
        if [ -z "$2" ]; then
            echo "Usage: $0 slug <market-slug>"
            exit 1
        fi
        fetch_slug "$2"
        ;;
    *)
        echo "Usage:"
        echo "  $0 bitcoin    - Search for Bitcoin markets"
        echo "  $0 ethereum   - Search for Ethereum markets"
        echo "  $0 crypto     - Search for both Bitcoin and Ethereum"
        echo "  $0 top        - Get top 20 markets by volume with Rust config"
        echo "  $0 slug <name> - Fetch specific market by slug"
        echo ""
        echo "Running default: top markets..."
        echo ""
        MARKETS=$(fetch_top_markets)
        echo "$MARKETS" | jq '.'
        output_rust_config "$MARKETS"
        ;;
esac
