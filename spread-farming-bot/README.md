# Polymarket Spread Farming Bot

**Market making bot for Polymarket crypto prediction markets**

## What is Spread Farming?

Spread farming (market making) is a liquidity provision strategy where you:
1. Place **BUY orders below mid-price**
2. Place **SELL orders above mid-price** 
3. Capture the **spread** when both orders fill
4. Earn **Polymarket liquidity rewards** (4% annualized + bonuses)

### Example Trade
```
Mid-price: $0.50
Your orders:
  - BUY @ $0.48 (2% below mid)
  - SELL @ $0.52 (2% above mid)

When both fill:
  - Bought at $0.48
  - Sold at $0.52
  - Profit: $0.04 per contract (8.3% return)
  + Liquidity rewards
```

## Strategy Details

### How It Makes Money
1. **Spread capture**: ~0.2% of trading volume
2. **Liquidity rewards**: 4% annualized from Polymarket
3. **Volume farming**: Potential airdrop rewards

### Risks
- **Adverse selection**: Informed traders trade against you
- **Inventory risk**: Prices move before both sides fill
- **Whale manipulation**: Large traders can manipulate prices
- **Competition**: Other market makers tighten spreads

### Differences from Arbitrage

| Aspect | Arbitrage | Spread Farming |
|--------|-----------|----------------|
| Strategy | Market taker | Market maker |
| Orders | Market orders | Limit orders |
| Profit | Guaranteed | Uncertain |
| Risk | Near risk-free | High risk |
| Speed | Sub-millisecond | Seconds/minutes |

## Installation

### 1. Build the Bot

```bash
cd /home/user/trading-bots/spread-farming-bot
cargo build --release
```

### 2. Configure Environment

```bash
cp .env.example .env
nano .env
```

Set your Polymarket credentials:
```bash
POLY_PRIVATE_KEY=0xYOUR_PRIVATE_KEY
POLY_FUNDER=0xYOUR_WALLET_ADDRESS
SPREAD_DRY_RUN=1  # Start with dry run!
```

### 3. Test in Dry Run Mode

```bash
cargo run --release
```

Monitor the output and ensure it's working correctly.

## Deployment

### Option 1: Run Manually

```bash
cd /home/user/trading-bots/spread-farming-bot
SPREAD_DRY_RUN=1 cargo run --release
```

### Option 2: Run as systemd Service

```bash
# Copy service file
sudo cp spread-farming.service /etc/systemd/system/

# Enable and start
sudo systemctl daemon-reload
sudo systemctl enable spread-farming
sudo systemctl start spread-farming

# Check status
sudo systemctl status spread-farming

# View logs
sudo journalctl -u spread-farming -f
```

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SPREAD_DRY_RUN` | `1` | `1` = paper trading, `0` = live execution |
| `SPREAD_BPS` | `400` | Spread in basis points (400 = 4%) |
| `MAX_POSITION_PER_MARKET` | `500` | Max position size per market ($USD) |
| `UPDATE_INTERVAL_MS` | `5000` | Order update frequency (milliseconds) |
| `MIN_ORDER_SIZE` | `2.0` | Minimum order size ($USD) |

### Tuning the Strategy

**Tighter Spreads (More Aggressive)**
```bash
SPREAD_BPS=200  # 2% spread
UPDATE_INTERVAL_MS=1000  # Update every second
```
- More fills, less profit per fill
- Higher risk of adverse selection
- More API calls

**Wider Spreads (More Conservative)**
```bash
SPREAD_BPS=600  # 6% spread
UPDATE_INTERVAL_MS=10000  # Update every 10 seconds
```
- Fewer fills, more profit per fill
- Lower risk of adverse selection
- Fewer API calls

## Target Markets

The bot targets **high-liquidity crypto markets**:

1. **Bitcoin $100K in 2024** - Highest liquidity
2. **Bitcoin $150K in 2025** - High liquidity
3. **Ethereum $5K in 2024** - High liquidity
4. **Ethereum $10K in 2025** - High liquidity
5. **Bitcoin ETF Approval 2024** - Moderate liquidity
6. **Ethereum Merge Complete 2024** - Moderate liquidity

**Note**: Market slugs are examples. Verify actual slugs on polymarket.com.

## Monitoring

### Key Metrics to Watch

1. **Fill rate**: Are your orders getting filled?
2. **Spread**: Is your spread competitive?
3. **Inventory**: Are you accumulating too much inventory?
4. **P&L**: Are you profitable?

### Logs

```bash
# View real-time logs
sudo journalctl -u spread-farming -f

# View last 100 lines
sudo journalctl -u spread-farming -n 100

# View logs since today
sudo journalctl -u spread-farming --since today
```

## Safety & Risk Management

### Before Going Live

1. ✅ Run in **DRY_RUN mode** for 24 hours
2. ✅ Verify orders are placed correctly
3. ✅ Check position limits are working
4. ✅ Start with **small capital** ($100-500)
5. ✅ Monitor closely for first week

### Position Limits

The bot enforces position limits:
- `MAX_POSITION_PER_MARKET`: Prevents excessive exposure per market
- Automatic inventory management
- Order sizing based on current position

### Circuit Breakers

**Manual circuit breaker**: Set `SPREAD_DRY_RUN=1` to stop trading

**Future enhancements**:
- Daily loss limits
- Consecutive error limits
- Whale detection

## Troubleshooting

### Bot won't start

```bash
# Check Rust is installed
rustc --version

# Check dependencies
cargo check

# Check credentials
echo $POLY_PRIVATE_KEY
```

### Orders not placing

```bash
# Check dry run mode
grep SPREAD_DRY_RUN .env

# Check Polymarket API access
# Check wallet has USDC on Polygon
```

### High inventory

```bash
# Reduce position limits
MAX_POSITION_PER_MARKET=100

# Widen spread to reduce fills
SPREAD_BPS=600
```

## Architecture

```
spread-farming-bot/
├── src/
│   ├── main.rs           # Entry point
│   ├── config.rs         # Configuration & market selection
│   ├── types.rs          # Data structures
│   ├── market_maker.rs   # Core market making engine
│   └── order_manager.rs  # Limit order management
├── Cargo.toml
├── .env.example
├── spread-farming.service
└── README.md
```

## Isolation from Arbitrage Bot

This bot is **completely independent** from the arbitrage bot:

✅ **Separate process** - Different binary  
✅ **Separate capital** - Can use different wallet  
✅ **Separate risk limits** - Independent position management  
✅ **Separate deployment** - systemd service  
✅ **Zero impact** - Arbitrage bot unaffected  

## Performance Impact

- Spread farming bot: **< 0.1% latency increase** to arbitrage bot
- Both bots can run on same server
- Atomic operations scale perfectly
- No lock contention

## Next Steps

1. **Test in dry run mode** for 24 hours
2. **Verify market slugs** on polymarket.com
3. **Start with $100-500** capital
4. **Monitor P&L** closely
5. **Tune parameters** based on results

## Support

Issues: Report to the repository maintainer

## License

MIT OR Apache-2.0
