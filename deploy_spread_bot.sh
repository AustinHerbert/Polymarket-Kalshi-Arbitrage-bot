#!/bin/bash
# Deploy spread farming bot to server
# Run this from your LOCAL machine (not on server)

set -e  # Exit on error

SERVER="root@167.172.24.118"
REMOTE_DIR="/root/trading-bots"
LOCAL_DIR="/home/user/trading-bots"

echo "=== Deploying Spread Farming Bot to Server ==="
echo ""

# Upload shared-market-client
echo "Uploading shared-market-client..."
ssh $SERVER "mkdir -p $REMOTE_DIR/shared-market-client/src"
scp "$LOCAL_DIR/shared-market-client/Cargo.toml" "$SERVER:$REMOTE_DIR/shared-market-client/"
scp "$LOCAL_DIR/shared-market-client/src/lib.rs" "$SERVER:$REMOTE_DIR/shared-market-client/src/"
scp "$LOCAL_DIR/shared-market-client/src/polymarket_clob.rs" "$SERVER:$REMOTE_DIR/shared-market-client/src/"

# Upload spread-farming-bot
echo "Uploading spread-farming-bot..."
ssh $SERVER "mkdir -p $REMOTE_DIR/spread-farming-bot/src"
scp "$LOCAL_DIR/spread-farming-bot/Cargo.toml" "$SERVER:$REMOTE_DIR/spread-farming-bot/"
scp "$LOCAL_DIR/spread-farming-bot/.env" "$SERVER:$REMOTE_DIR/spread-farming-bot/"
scp "$LOCAL_DIR/spread-farming-bot/src/main.rs" "$SERVER:$REMOTE_DIR/spread-farming-bot/src/"
scp "$LOCAL_DIR/spread-farming-bot/src/config.rs" "$SERVER:$REMOTE_DIR/spread-farming-bot/src/"
scp "$LOCAL_DIR/spread-farming-bot/src/types.rs" "$SERVER:$REMOTE_DIR/spread-farming-bot/src/"
scp "$LOCAL_DIR/spread-farming-bot/src/order_manager.rs" "$SERVER:$REMOTE_DIR/spread-farming-bot/src/"
scp "$LOCAL_DIR/spread-farming-bot/src/market_maker.rs" "$SERVER:$REMOTE_DIR/spread-farming-bot/src/"

echo ""
echo "=== Building on server ==="
ssh $SERVER "cd $REMOTE_DIR/spread-farming-bot && cargo build --release"

echo ""
echo "=== DEPLOYMENT COMPLETE ==="
echo ""
echo "To test the bot, SSH to server and run:"
echo "  ssh $SERVER"
echo "  cd $REMOTE_DIR/spread-farming-bot"
echo "  RUST_LOG=info ./target/release/spread-farming-bot"
