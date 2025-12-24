#!/bin/bash
# Script to update spread farming bot files on server

SERVER="root@167.172.24.118"
REMOTE_DIR="/root/trading-bots"
LOCAL_DIR="/home/user/trading-bots"

echo "Uploading updated files to server..."

# Upload shared-market-client updates
scp "$LOCAL_DIR/shared-market-client/src/polymarket_clob.rs" "$SERVER:$REMOTE_DIR/shared-market-client/src/"
scp "$LOCAL_DIR/shared-market-client/src/lib.rs" "$SERVER:$REMOTE_DIR/shared-market-client/src/"

# Upload spread-farming-bot updates
scp "$LOCAL_DIR/spread-farming-bot/src/config.rs" "$SERVER:$REMOTE_DIR/spread-farming-bot/src/"
scp "$LOCAL_DIR/spread-farming-bot/src/market_maker.rs" "$SERVER:$REMOTE_DIR/spread-farming-bot/src/"
scp "$LOCAL_DIR/spread-farming-bot/.env" "$SERVER:$REMOTE_DIR/spread-farming-bot/"

echo "Files uploaded. Now SSH to server and rebuild:"
echo "  ssh $SERVER"
echo "  cd $REMOTE_DIR/spread-farming-bot"
echo "  cargo build --release"
echo "  RUST_LOG=info ./target/release/spread-farming-bot"
