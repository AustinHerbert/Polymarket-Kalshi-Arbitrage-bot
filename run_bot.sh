#!/bin/bash
# Auto-restart wrapper for the arbitrage bot
# Usage: ./run_bot.sh
# Or with nohup: nohup ./run_bot.sh > bot.log 2>&1 &

cd "$(dirname "$0")"

echo "Starting arbitrage bot with auto-restart..."
echo "Press Ctrl+C to stop completely"

while true; do
    echo "[$(date)] Starting bot..."
    ./target/release/prediction-market-arbitrage

    EXIT_CODE=$?
    echo "[$(date)] Bot exited with code $EXIT_CODE"

    if [ $EXIT_CODE -eq 0 ]; then
        echo "[$(date)] Clean exit (restart requested) - restarting in 3 seconds..."
        sleep 3
    else
        echo "[$(date)] Unexpected exit - restarting in 10 seconds..."
        sleep 10
    fi
done
