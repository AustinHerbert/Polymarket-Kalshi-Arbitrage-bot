#!/bin/bash

echo "=========================================="
echo "Bot Performance Comparison"
echo "=========================================="
echo ""

# Find the most recent log files
MY_BOT_LOG=$(ls -t logs/my-bot_*.log 2>/dev/null | head -1)
ORIGINAL_BOT_LOG=$(ls -t logs/original-bot_*.log 2>/dev/null | head -1)

if [ -z "$MY_BOT_LOG" ] || [ -z "$ORIGINAL_BOT_LOG" ]; then
    echo "Error: Log files not found!"
    echo "Make sure you've run ./run_both_bots.sh first"
    exit 1
fi

echo "Analyzing logs:"
echo "  Your bot:     $MY_BOT_LOG"
echo "  Original bot: $ORIGINAL_BOT_LOG"
echo ""
echo "=========================================="
echo ""

# Function to analyze a log file
analyze_log() {
    local LOG_FILE=$1
    local BOT_NAME=$2

    echo "[$BOT_NAME]"
    echo "─────────────────────────────────────────"

    # Count arbitrage opportunities detected
    local OPPS_DETECTED=$(grep -c "\[EXEC\] 🎯" "$LOG_FILE" 2>/dev/null || echo "0")
    echo "  Opportunities detected:     $OPPS_DETECTED"

    # Count successful executions (dry run or real)
    local EXECUTIONS=$(grep -c "DRY RUN - would execute\|✅ market_id=" "$LOG_FILE" 2>/dev/null || echo "0")
    echo "  Would execute (DRY_RUN):    $EXECUTIONS"

    # Extract profit values and calculate total
    local TOTAL_PROFIT=$(grep "\[EXEC\] 🎯" "$LOG_FILE" | grep -o "profit=[0-9]*¢" | grep -o "[0-9]*" | awk '{sum+=$1} END {print sum}')
    [ -z "$TOTAL_PROFIT" ] && TOTAL_PROFIT=0
    echo "  Total potential profit:     ${TOTAL_PROFIT}¢ (\$$(echo "scale=2; $TOTAL_PROFIT/100" | bc))"

    # Calculate average profit per opportunity
    if [ "$OPPS_DETECTED" -gt 0 ]; then
        local AVG_PROFIT=$(echo "scale=2; $TOTAL_PROFIT / $OPPS_DETECTED" | bc)
        echo "  Avg profit per opportunity: ${AVG_PROFIT}¢"
    fi

    # Count arbitrage types
    local POLY_YES_KALSHI_NO=$(grep "\[EXEC\] 🎯" "$LOG_FILE" | grep -c "PolyYesKalshiNo" 2>/dev/null || echo "0")
    local KALSHI_YES_POLY_NO=$(grep "\[EXEC\] 🎯" "$LOG_FILE" | grep -c "KalshiYesPolyNo" 2>/dev/null || echo "0")
    local POLY_ONLY=$(grep "\[EXEC\] 🎯" "$LOG_FILE" | grep -c "PolyOnly" 2>/dev/null || echo "0")
    local KALSHI_ONLY=$(grep "\[EXEC\] 🎯" "$LOG_FILE" | grep -c "KalshiOnly" 2>/dev/null || echo "0")

    echo ""
    echo "  Opportunity breakdown:"
    echo "    Poly YES + Kalshi NO:     $POLY_YES_KALSHI_NO"
    echo "    Kalshi YES + Poly NO:     $KALSHI_YES_POLY_NO"
    echo "    Poly-Poly (same market):  $POLY_ONLY"
    echo "    Kalshi-Kalshi:            $KALSHI_ONLY"

    # Extract latency stats
    echo ""
    echo "  Performance:"
    local AVG_LATENCY=$(grep "\[EXEC\] 🎯" "$LOG_FILE" | grep -o "[0-9]*µs" | grep -o "[0-9]*" | awk '{sum+=$1; count++} END {if(count>0) print sum/count; else print 0}')
    [ -z "$AVG_LATENCY" ] && AVG_LATENCY=0
    echo "    Avg detection latency:    ${AVG_LATENCY}µs"

    # Count errors/warnings
    local ERRORS=$(grep -c "\[EXEC\] ❌" "$LOG_FILE" 2>/dev/null || echo "0")
    local WARNINGS=$(grep -c "\[EXEC\] ⚠️" "$LOG_FILE" 2>/dev/null || echo "0")
    echo "    Errors:                   $ERRORS"
    echo "    Warnings:                 $WARNINGS"

    echo ""
}

# Analyze both bots
analyze_log "$MY_BOT_LOG" "YOUR BOT"
echo ""
analyze_log "$ORIGINAL_BOT_LOG" "ORIGINAL BOT"
echo ""

echo "=========================================="
echo "Position Files Comparison"
echo "=========================================="
echo ""

if [ -f "positions.json" ]; then
    echo "[YOUR BOT] positions.json"
    echo "─────────────────────────────────────────"
    jq -r '.positions | length as $count | .daily_realized_pnl as $daily | .all_time_pnl as $total | "  Open positions: \($count)\n  Daily P&L: $\($daily)\n  All-time P&L: $\($total)"' positions.json 2>/dev/null || echo "  Unable to parse positions.json"
    echo ""
fi

if [ -f "original-bot/positions.json" ]; then
    echo "[ORIGINAL BOT] original-bot/positions.json"
    echo "─────────────────────────────────────────"
    jq -r '.positions | length as $count | .daily_realized_pnl as $daily | .all_time_pnl as $total | "  Open positions: \($count)\n  Daily P&L: $\($daily)\n  All-time P&L: $\($total)"' original-bot/positions.json 2>/dev/null || echo "  Unable to parse positions.json"
    echo ""
fi

echo "=========================================="
echo ""
echo "For detailed analysis, view the full logs:"
echo "  less $MY_BOT_LOG"
echo "  less $ORIGINAL_BOT_LOG"
echo ""
echo "To see live differences:"
echo "  diff -u <(grep '🎯' $MY_BOT_LOG) <(grep '🎯' $ORIGINAL_BOT_LOG)"
echo ""
