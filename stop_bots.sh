#!/bin/bash

echo "Stopping both bots..."

# Kill screen sessions
screen -S my-bot -X quit 2>/dev/null && echo "✓ Stopped my-bot" || echo "  my-bot not running"
screen -S original-bot -X quit 2>/dev/null && echo "✓ Stopped original-bot" || echo "  original-bot not running"

echo ""
echo "All bots stopped."
echo "Logs are preserved in the logs/ directory"
echo "Use ./compare_bots.sh to analyze results"
