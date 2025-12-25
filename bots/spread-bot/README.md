# Spread Bot

Market making / spread farming bot for Polymarket crypto prediction markets.

## Features

- **Liquidity provision**: Place limit orders on both sides
- **Spread capture**: Earn the bid-ask spread
- **Verified token IDs**: Real Polymarket market tokens (Dec 2025)

## Target Markets

| Market | Token IDs |
|--------|-----------|
| BTC $95K | 96867039...698980 |
| BTC $75K dip | 43814376...628422 |
| ETH $5K | 96638575...427384 |

## Usage

```bash
# Build
cargo build --release -p spread-bot

# Dry run
SPREAD_DRY_RUN=1 ./target/release/spread-bot

# Live (careful!)
SPREAD_DRY_RUN=0 ./target/release/spread-bot
```

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| SPREAD_BPS | 200 | Spread in basis points (2%) |
| MAX_POSITION_PER_MARKET | 50 | Max USD per market |
| UPDATE_INTERVAL_MS | 10000 | Quote update frequency |
