#!/usr/bin/env python3
"""
Parameter Tuning Tool for Arbitrage Bot

This script runs the bot with different parameter configurations and compares results.
Useful for finding optimal settings for your trading strategy.

Usage:
    python3 scripts/tune_parameters.py --duration 60 --dry-run
    python3 scripts/tune_parameters.py --compare metrics_standard_latest.json metrics_priority_latest.json
    python3 scripts/tune_parameters.py --sweep
"""

import argparse
import json
import os
import subprocess
import sys
import time
from dataclasses import dataclass, asdict
from datetime import datetime
from pathlib import Path
from typing import Optional, List, Dict, Any


@dataclass
class PriorityConfig:
    """Priority mode configuration parameters"""
    min_liquidity_cents: int = 25000      # $250
    max_liquidity_cents: int = 250000     # $2500
    min_arb_percent: float = 1.0
    queue_sort_interval_secs: int = 5
    live_priority_boost: float = 10.0
    expiration_weight: float = 1.0
    profit_weight: float = 2.0

    def to_env(self) -> Dict[str, str]:
        """Convert to environment variables"""
        return {
            "PRIORITY_MODE": "1",
            "MIN_LIQUIDITY_CENTS": str(self.min_liquidity_cents),
            "MAX_LIQUIDITY_CENTS": str(self.max_liquidity_cents),
            "MIN_ARB_PERCENT": str(self.min_arb_percent),
            "QUEUE_SORT_INTERVAL_SECS": str(self.queue_sort_interval_secs),
            "LIVE_PRIORITY_BOOST": str(self.live_priority_boost),
            "EXPIRATION_WEIGHT": str(self.expiration_weight),
            "PROFIT_WEIGHT": str(self.profit_weight),
        }

    def description(self) -> str:
        """Human-readable description"""
        return (f"MinLiq=${self.min_liquidity_cents//100}, "
                f"MaxLiq=${self.max_liquidity_cents//100}, "
                f"MinArb={self.min_arb_percent}%, "
                f"LiveBoost={self.live_priority_boost}x")


@dataclass
class MetricsSnapshot:
    """Metrics snapshot from the bot"""
    priority_mode: bool = False
    duration_secs: int = 0
    opportunities_detected: int = 0
    trades_executed: int = 0
    trades_rejected: int = 0
    rejected_low_profit: int = 0
    rejected_low_liquidity: int = 0
    trades_clamped: int = 0
    live_game_trades: int = 0
    total_profit_cents: int = 0
    total_volume_cents: int = 0
    total_fees_cents: int = 0
    avg_latency_ns: int = 0
    min_latency_ns: int = 0
    max_latency_ns: int = 0

    @classmethod
    def from_json(cls, path: str) -> "MetricsSnapshot":
        """Load from JSON file"""
        with open(path) as f:
            data = json.load(f)
        return cls(**data)

    def trades_per_hour(self) -> float:
        if self.duration_secs == 0:
            return 0
        return self.trades_executed / (self.duration_secs / 3600)

    def profit_per_hour(self) -> float:
        if self.duration_secs == 0:
            return 0
        return (self.total_profit_cents / 100) / (self.duration_secs / 3600)

    def rate_of_return(self) -> float:
        if self.total_volume_cents == 0:
            return 0
        return (self.total_profit_cents / self.total_volume_cents) * 100

    def conversion_rate(self) -> float:
        if self.opportunities_detected == 0:
            return 0
        return (self.trades_executed / self.opportunities_detected) * 100


def print_header(text: str):
    """Print a formatted header"""
    width = 70
    print("=" * width)
    print(f" {text}".center(width))
    print("=" * width)


def print_comparison(std: MetricsSnapshot, pri: MetricsSnapshot):
    """Print side-by-side comparison"""
    print_header("COMPARISON: Standard vs Priority Mode")
    print()

    std_hours = std.duration_secs / 3600
    pri_hours = pri.duration_secs / 3600

    def fmt_diff(std_val: float, pri_val: float, is_pct: bool = False) -> str:
        diff = pri_val - std_val
        sign = "+" if diff > 0 else ""
        if is_pct:
            return f"{sign}{diff:.2f}%"
        elif abs(diff) >= 1:
            return f"{sign}{diff:.1f}"
        else:
            return f"{sign}{diff:.2f}"

    def pct_change(old: float, new: float) -> str:
        if old == 0:
            return "+∞%" if new > 0 else "0%"
        pct = ((new - old) / abs(old)) * 100
        return f"{pct:+.1f}%"

    rows = [
        ("Duration", f"{std_hours:.1f}h", f"{pri_hours:.1f}h", "-", "-"),
        ("Trades Executed", str(std.trades_executed), str(pri.trades_executed),
         fmt_diff(std.trades_executed, pri.trades_executed),
         pct_change(std.trades_executed, pri.trades_executed)),
        ("Trade Rate (/hr)", f"{std.trades_per_hour():.1f}", f"{pri.trades_per_hour():.1f}",
         fmt_diff(std.trades_per_hour(), pri.trades_per_hour()),
         pct_change(std.trades_per_hour(), pri.trades_per_hour())),
        ("Total Profit ($)", f"{std.total_profit_cents/100:.2f}", f"{pri.total_profit_cents/100:.2f}",
         fmt_diff(std.total_profit_cents/100, pri.total_profit_cents/100),
         pct_change(std.total_profit_cents, pri.total_profit_cents)),
        ("Profit/Hour ($)", f"{std.profit_per_hour():.2f}", f"{pri.profit_per_hour():.2f}",
         fmt_diff(std.profit_per_hour(), pri.profit_per_hour()),
         pct_change(std.profit_per_hour(), pri.profit_per_hour())),
        ("Total Volume ($)", f"{std.total_volume_cents/100:.0f}", f"{pri.total_volume_cents/100:.0f}",
         fmt_diff(std.total_volume_cents/100, pri.total_volume_cents/100),
         pct_change(std.total_volume_cents, pri.total_volume_cents)),
        ("Rate of Return (%)", f"{std.rate_of_return():.2f}", f"{pri.rate_of_return():.2f}",
         fmt_diff(std.rate_of_return(), pri.rate_of_return(), True), "-"),
        ("Trades Rejected", str(std.trades_rejected), str(pri.trades_rejected),
         fmt_diff(std.trades_rejected, pri.trades_rejected),
         pct_change(std.trades_rejected, pri.trades_rejected)),
        ("Live Game Trades", "-", str(pri.live_game_trades), "-", "-"),
        ("Avg Latency (ms)", f"{std.avg_latency_ns/1e6:.1f}", f"{pri.avg_latency_ns/1e6:.1f}",
         fmt_diff(std.avg_latency_ns/1e6, pri.avg_latency_ns/1e6),
         pct_change(std.avg_latency_ns, pri.avg_latency_ns)),
    ]

    print(f"{'METRIC':<25} {'STANDARD':>12} {'PRIORITY':>12} {'DIFF':>12} {'%CHANGE':>10}")
    print("-" * 70)

    for row in rows:
        label, std_val, pri_val, diff, pct = row
        # Color coding for terminal
        diff_color = ""
        if diff not in ["-", ""]:
            try:
                if float(diff.replace("+", "").replace("%", "")) > 0:
                    diff_color = "\033[92m"  # Green
                elif float(diff.replace("+", "").replace("%", "")) < 0:
                    diff_color = "\033[91m"  # Red
            except:
                pass
        reset = "\033[0m" if diff_color else ""
        print(f"{label:<25} {std_val:>12} {pri_val:>12} {diff_color}{diff:>12}{reset} {pct:>10}")

    print()
    print("=" * 70)

    # Summary
    profit_improvement = 0
    if std.total_profit_cents > 0:
        profit_improvement = ((pri.total_profit_cents - std.total_profit_cents) / std.total_profit_cents) * 100

    print()
    print("SUMMARY:")
    print(f"  Profit improvement: {profit_improvement:+.1f}%")
    if pri.total_profit_cents > std.total_profit_cents:
        print("  \033[92m✓ Priority mode IS more profitable\033[0m")
    else:
        print("  \033[91m✗ Priority mode is NOT more profitable\033[0m")
    print()


def run_bot(duration_secs: int, config: Optional[PriorityConfig] = None, dry_run: bool = True) -> Optional[str]:
    """Run the bot for specified duration and return metrics file path"""
    env = os.environ.copy()
    env["DRY_RUN"] = "1" if dry_run else "0"

    mode = "standard"
    if config:
        env.update(config.to_env())
        mode = "priority"
    else:
        env["PRIORITY_MODE"] = "0"

    print(f"\n[*] Starting {mode} mode for {duration_secs}s...")
    if config:
        print(f"    Config: {config.description()}")

    try:
        # Build if needed
        subprocess.run(["cargo", "build", "--release"], check=True, capture_output=True)

        # Run bot with timeout
        result = subprocess.run(
            ["./target/release/prediction-market-arbitrage"],
            env=env,
            timeout=duration_secs + 10,
            capture_output=True,
            text=True
        )
    except subprocess.TimeoutExpired:
        pass  # Expected
    except subprocess.CalledProcessError as e:
        print(f"[!] Error running bot: {e}")
        return None
    except FileNotFoundError:
        print("[!] Bot executable not found. Run 'cargo build --release' first.")
        return None

    metrics_file = f"metrics_{mode}_latest.json"
    if os.path.exists(metrics_file):
        print(f"[+] Metrics saved to {metrics_file}")
        return metrics_file

    return None


def run_sweep(duration_per_config: int = 300, dry_run: bool = True):
    """Run a parameter sweep to find optimal settings"""
    print_header("PARAMETER SWEEP")

    # Define parameter ranges to test
    configs = [
        # Vary min liquidity
        PriorityConfig(min_liquidity_cents=10000),   # $100
        PriorityConfig(min_liquidity_cents=25000),   # $250 (default)
        PriorityConfig(min_liquidity_cents=50000),   # $500

        # Vary min arb percent
        PriorityConfig(min_arb_percent=0.5),
        PriorityConfig(min_arb_percent=1.0),  # default
        PriorityConfig(min_arb_percent=2.0),

        # Vary live priority boost
        PriorityConfig(live_priority_boost=5.0),
        PriorityConfig(live_priority_boost=10.0),  # default
        PriorityConfig(live_priority_boost=20.0),
    ]

    results = []
    results_dir = Path(f"sweep_results_{datetime.now().strftime('%Y%m%d_%H%M%S')}")
    results_dir.mkdir(exist_ok=True)

    # First run standard mode as baseline
    print("\n[1/{}] Running baseline (standard mode)...".format(len(configs) + 1))
    std_file = run_bot(duration_per_config, config=None, dry_run=dry_run)
    if std_file:
        std_metrics = MetricsSnapshot.from_json(std_file)
        results.append(("Standard", None, std_metrics))

    # Run each configuration
    for i, config in enumerate(configs, 2):
        print(f"\n[{i}/{len(configs)+1}] Testing: {config.description()}")
        metrics_file = run_bot(duration_per_config, config=config, dry_run=dry_run)

        if metrics_file:
            metrics = MetricsSnapshot.from_json(metrics_file)
            results.append((config.description(), config, metrics))

            # Save individual result
            result_file = results_dir / f"config_{i-1}.json"
            with open(result_file, 'w') as f:
                json.dump({
                    "config": asdict(config),
                    "metrics": asdict(metrics)
                }, f, indent=2)

        # Small delay between runs
        time.sleep(5)

    # Print summary
    print_header("SWEEP RESULTS SUMMARY")
    print()
    print(f"{'CONFIG':<40} {'TRADES':>8} {'PROFIT':>10} {'$/HR':>8} {'RoR':>8}")
    print("-" * 80)

    for name, config, metrics in results:
        print(f"{name:<40} {metrics.trades_executed:>8} ${metrics.total_profit_cents/100:>9.2f} "
              f"${metrics.profit_per_hour():>7.2f} {metrics.rate_of_return():>7.2f}%")

    # Find best configuration
    if len(results) > 1:
        best = max(results[1:], key=lambda x: x[2].profit_per_hour())
        print()
        print(f"BEST CONFIG: {best[0]}")
        print(f"  Profit/hour: ${best[2].profit_per_hour():.2f}")
        print(f"  Rate of Return: {best[2].rate_of_return():.2f}%")

    # Save summary
    summary_file = results_dir / "summary.json"
    with open(summary_file, 'w') as f:
        json.dump([
            {
                "name": name,
                "config": asdict(config) if config else None,
                "profit_per_hour": metrics.profit_per_hour(),
                "rate_of_return": metrics.rate_of_return(),
                "trades_executed": metrics.trades_executed
            }
            for name, config, metrics in results
        ], f, indent=2)

    print(f"\n[+] Results saved to {results_dir}/")


def main():
    parser = argparse.ArgumentParser(
        description="Parameter tuning tool for the arbitrage bot",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Compare two existing metrics files
  python3 scripts/tune_parameters.py --compare metrics_standard_latest.json metrics_priority_latest.json

  # Run a parameter sweep (5 minutes per config)
  python3 scripts/tune_parameters.py --sweep --duration 300 --dry-run

  # Run a quick test with specific config
  python3 scripts/tune_parameters.py --test --min-liq 500 --min-arb 1.5 --duration 60
        """
    )

    parser.add_argument("--compare", nargs=2, metavar=("STD_FILE", "PRI_FILE"),
                        help="Compare two metrics JSON files")
    parser.add_argument("--sweep", action="store_true",
                        help="Run a parameter sweep to find optimal settings")
    parser.add_argument("--test", action="store_true",
                        help="Run a single test with specified parameters")
    parser.add_argument("--duration", type=int, default=300,
                        help="Duration per configuration in seconds (default: 300)")
    parser.add_argument("--dry-run", action="store_true", default=True,
                        help="Run in dry-run mode (default: True)")
    parser.add_argument("--live", action="store_true",
                        help="Run in live mode (DANGER: will execute real trades)")

    # Config options for --test
    parser.add_argument("--min-liq", type=int, default=250,
                        help="Minimum liquidity in dollars (default: 250)")
    parser.add_argument("--max-liq", type=int, default=2500,
                        help="Maximum liquidity in dollars (default: 2500)")
    parser.add_argument("--min-arb", type=float, default=1.0,
                        help="Minimum arb percent (default: 1.0)")
    parser.add_argument("--live-boost", type=float, default=10.0,
                        help="Live game priority boost (default: 10.0)")

    args = parser.parse_args()

    if args.live:
        print("\033[91m" + "=" * 60)
        print(" WARNING: LIVE MODE - REAL TRADES WILL BE EXECUTED!")
        print("=" * 60 + "\033[0m")
        confirm = input("Type 'CONFIRM' to proceed: ")
        if confirm != "CONFIRM":
            print("Aborted.")
            sys.exit(1)
        args.dry_run = False

    if args.compare:
        std_file, pri_file = args.compare
        if not os.path.exists(std_file):
            print(f"Error: File not found: {std_file}")
            sys.exit(1)
        if not os.path.exists(pri_file):
            print(f"Error: File not found: {pri_file}")
            sys.exit(1)

        std = MetricsSnapshot.from_json(std_file)
        pri = MetricsSnapshot.from_json(pri_file)
        print_comparison(std, pri)

    elif args.sweep:
        run_sweep(duration_per_config=args.duration, dry_run=args.dry_run)

    elif args.test:
        config = PriorityConfig(
            min_liquidity_cents=args.min_liq * 100,
            max_liquidity_cents=args.max_liq * 100,
            min_arb_percent=args.min_arb,
            live_priority_boost=args.live_boost
        )
        print_header("SINGLE CONFIG TEST")
        print(f"Config: {config.description()}")
        print(f"Duration: {args.duration}s")
        print(f"Mode: {'DRY RUN' if args.dry_run else 'LIVE'}")

        # Run standard mode first
        std_file = run_bot(args.duration, config=None, dry_run=args.dry_run)

        # Then priority mode
        pri_file = run_bot(args.duration, config=config, dry_run=args.dry_run)

        if std_file and pri_file:
            std = MetricsSnapshot.from_json(std_file)
            pri = MetricsSnapshot.from_json(pri_file)
            print_comparison(std, pri)

    else:
        parser.print_help()


if __name__ == "__main__":
    main()
