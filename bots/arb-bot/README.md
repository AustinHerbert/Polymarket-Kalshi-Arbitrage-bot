# Arb Bot

Cross-platform arbitrage bot for Kalshi and Polymarket prediction markets.

## Features

- **Kalshi-Poly arbitrage**: Cross-platform opportunities
- **Poly-Poly arbitrage**: Same-platform Polymarket opportunities
- **Kalshi-Kalshi arbitrage**: Same-platform Kalshi opportunities
- **Sub-millisecond latency**: Lock-free atomic orderbook cache
- **SIMD-accelerated**: Fast arbitrage detection

## Usage

```bash
# Build
cargo build --release -p arb-bot

# Dry run (paper trading)
DRY_RUN=1 ./target/release/arb-bot

# Live trading
DRY_RUN=0 ./target/release/arb-bot
```

## Required Environment Variables

```bash
KALSHI_API_KEY_ID=your_api_key
KALSHI_PRIVATE_KEY_PATH=/path/to/key.pem
POLY_PRIVATE_KEY=0x...
POLY_FUNDER=0x...
```
