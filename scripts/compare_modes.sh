#!/bin/bash
#
# Compare Standard vs Priority Mode Performance
#
# This script helps you run the bot in both modes and compare metrics.
#
# Usage:
#   ./scripts/compare_modes.sh [duration_hours]
#
# Example:
#   ./scripts/compare_modes.sh 24   # Run each mode for 24 hours
#

set -e

DURATION_HOURS=${1:-1}
DURATION_SECS=$((DURATION_HOURS * 3600))

echo "═══════════════════════════════════════════════════════════════"
echo "📊 MODE COMPARISON TEST"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Duration per mode: ${DURATION_HOURS} hour(s)"
echo ""

# Build the project
echo "Building project..."
cargo build --release
echo ""

# Create results directory
RESULTS_DIR="comparison_results_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$RESULTS_DIR"

echo "Results will be saved to: $RESULTS_DIR"
echo ""

# Function to run a test
run_test() {
    local mode=$1
    local priority_mode=$2
    local output_file="$RESULTS_DIR/${mode}_output.log"

    echo "═══════════════════════════════════════════════════════════════"
    echo "🚀 Starting $mode mode test..."
    echo "═══════════════════════════════════════════════════════════════"

    # Set environment variables
    export PRIORITY_MODE=$priority_mode
    export DRY_RUN=1

    # Run for specified duration
    timeout "${DURATION_SECS}s" ./target/release/prediction-market-arbitrage 2>&1 | tee "$output_file" || true

    # Copy metrics file
    if [ -f "metrics_${mode}_latest.json" ]; then
        cp "metrics_${mode}_latest.json" "$RESULTS_DIR/"
        echo "✅ Metrics saved to $RESULTS_DIR/metrics_${mode}_latest.json"
    fi

    echo ""
}

# Run standard mode first
run_test "standard" "0"

echo ""
echo "Waiting 10 seconds before starting priority mode..."
sleep 10
echo ""

# Run priority mode
run_test "priority" "1"

# Compare results
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "📊 COMPARISON RESULTS"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Check if we have both metrics files
if [ -f "$RESULTS_DIR/metrics_standard_latest.json" ] && [ -f "$RESULTS_DIR/metrics_priority_latest.json" ]; then
    echo "Both metrics files found. Creating comparison..."

    # Create a simple comparison using jq (if available) or Python
    if command -v python3 &> /dev/null; then
        python3 << 'PYTHON_SCRIPT'
import json
import sys

def load_metrics(path):
    try:
        with open(path) as f:
            return json.load(f)
    except:
        return None

import os
results_dir = [d for d in os.listdir('.') if d.startswith('comparison_results_')][-1]

std = load_metrics(f'{results_dir}/metrics_standard_latest.json')
pri = load_metrics(f'{results_dir}/metrics_priority_latest.json')

if not std or not pri:
    print("Could not load metrics files")
    sys.exit(1)

print("=" * 75)
print(f"{'METRIC':<30} {'STANDARD':>15} {'PRIORITY':>15} {'DIFFERENCE':>15}")
print("-" * 75)

def fmt_diff(std_val, pri_val, is_money=False):
    diff = pri_val - std_val
    if is_money:
        return f"${diff:+.2f}"
    else:
        return f"{diff:+.1f}"

# Duration
std_hrs = std['duration_secs'] / 3600
pri_hrs = pri['duration_secs'] / 3600
print(f"{'Duration':<30} {std_hrs:>12.2f}h {pri_hrs:>12.2f}h {'-':>15}")

# Trades
print(f"{'Trades Executed':<30} {std['trades_executed']:>15} {pri['trades_executed']:>15} {pri['trades_executed'] - std['trades_executed']:>+15}")

# Trades per hour
std_tph = std['trades_executed'] / std_hrs if std_hrs > 0 else 0
pri_tph = pri['trades_executed'] / pri_hrs if pri_hrs > 0 else 0
print(f"{'Trade Rate':<30} {std_tph:>12.1f}/h {pri_tph:>12.1f}/h {pri_tph - std_tph:>+12.1f}/h")

# Profit
std_profit = std['total_profit_cents'] / 100
pri_profit = pri['total_profit_cents'] / 100
print(f"{'Total Profit':<30} ${std_profit:>13.2f} ${pri_profit:>13.2f} ${pri_profit - std_profit:>+13.2f}")

# Profit per hour
std_pph = std_profit / std_hrs if std_hrs > 0 else 0
pri_pph = pri_profit / pri_hrs if pri_hrs > 0 else 0
print(f"{'Profit Rate':<30} ${std_pph:>11.2f}/h ${pri_pph:>11.2f}/h ${pri_pph - std_pph:>+11.2f}/h")

# Volume
std_vol = std['total_volume_cents'] / 100
pri_vol = pri['total_volume_cents'] / 100
print(f"{'Total Volume':<30} ${std_vol:>13.2f} ${pri_vol:>13.2f} ${pri_vol - std_vol:>+13.2f}")

# Rate of return
std_ror = (std['total_profit_cents'] / std['total_volume_cents'] * 100) if std['total_volume_cents'] > 0 else 0
pri_ror = (pri['total_profit_cents'] / pri['total_volume_cents'] * 100) if pri['total_volume_cents'] > 0 else 0
print(f"{'Rate of Return':<30} {std_ror:>13.2f}% {pri_ror:>13.2f}% {pri_ror - std_ror:>+13.2f}%")

# Rejections
print(f"{'Trades Rejected':<30} {std['trades_rejected']:>15} {pri['trades_rejected']:>15} {pri['trades_rejected'] - std['trades_rejected']:>+15}")

# Live game trades
print(f"{'Live Game Trades':<30} {'-':>15} {pri['live_game_trades']:>15} {'(priority only)':>15}")

print("=" * 75)
print()

# Summary
if std_profit > 0:
    profit_improvement = ((pri_profit - std_profit) / std_profit * 100)
else:
    profit_improvement = 0

print("SUMMARY:")
print(f"  Profit improvement: {profit_improvement:+.1f}%")
print(f"  Priority mode {'IS' if pri_profit > std_profit else 'is NOT'} more profitable")
PYTHON_SCRIPT
    else
        echo "Python3 not found. Please compare the JSON files manually:"
        echo "  - $RESULTS_DIR/metrics_standard_latest.json"
        echo "  - $RESULTS_DIR/metrics_priority_latest.json"
    fi
else
    echo "⚠️  Metrics files not found. The bot may not have run long enough."
    echo "    Expected files:"
    echo "    - $RESULTS_DIR/metrics_standard_latest.json"
    echo "    - $RESULTS_DIR/metrics_priority_latest.json"
fi

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "✅ Comparison complete!"
echo "   Results saved to: $RESULTS_DIR"
echo "═══════════════════════════════════════════════════════════════"
