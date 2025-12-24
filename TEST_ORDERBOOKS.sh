#!/bin/bash
# Test orderbook API with real token IDs
# Run this to complete Step 3: Test orderbook API with real token IDs

set -e

SERVER="root@167.172.24.118"
REMOTE_DIR="/root/trading-bots"
LOCAL_DIR="/home/user/trading-bots"

echo "=== Step 3: Testing Orderbook API with Real Token IDs ==="
echo ""

# Upload test binary source
echo "1. Uploading test script to server..."
ssh $SERVER "mkdir -p $REMOTE_DIR/spread-farming-bot/src/bin"
scp "$LOCAL_DIR/spread-farming-bot/src/bin/test_orderbooks.rs" "$SERVER:$REMOTE_DIR/spread-farming-bot/src/bin/"
scp "$LOCAL_DIR/spread-farming-bot/Cargo.toml" "$SERVER:$REMOTE_DIR/spread-farming-bot/"

echo ""
echo "2. Building test binary on server..."
ssh $SERVER "cd $REMOTE_DIR/spread-farming-bot && cargo build --release --bin test-orderbooks"

echo ""
echo "3. Running orderbook API tests..."
echo ""
ssh $SERVER "cd $REMOTE_DIR/spread-farming-bot && ./target/release/test-orderbooks"

echo ""
echo "=== Test Complete ==="
echo ""
echo "Review the output above to:"
echo "  1. See which price outcomes have good liquidity"
echo "  2. Copy the token IDs for outcomes you want to market make"
echo "  3. Update src/config.rs with those token IDs"
