#!/bin/bash

echo "=========================================="
echo "Stopping All Bots and Monitor"
echo "=========================================="
echo ""

# Kill screen sessions
echo "Stopping screen sessions..."
screen -S my-bot -X quit 2>/dev/null && echo "  ✓ Stopped my-bot" || echo "  - my-bot not running"
screen -S original-bot -X quit 2>/dev/null && echo "  ✓ Stopped original-bot" || echo "  - original-bot not running"
screen -S bot -X quit 2>/dev/null && echo "  ✓ Stopped bot" || echo "  - bot not running"
screen -S monitor -X quit 2>/dev/null && echo "  ✓ Stopped monitor" || echo "  - monitor not running"
screen -S dashboard -X quit 2>/dev/null && echo "  ✓ Stopped dashboard" || echo "  - dashboard not running"

# Kill any remaining processes
echo ""
echo "Checking for remaining processes..."
PIDS=$(ps aux | grep -E 'prediction-market-arbitrage|target/release/monitor' | grep -v grep | awk '{print $2}')
if [ ! -z "$PIDS" ]; then
    echo "Killing remaining processes: $PIDS"
    echo "$PIDS" | xargs kill -9 2>/dev/null
    echo "  ✓ All processes terminated"
else
    echo "  - No remaining processes"
fi

echo ""
echo "=========================================="
echo "✓ All Services Stopped"
echo "=========================================="
echo ""
