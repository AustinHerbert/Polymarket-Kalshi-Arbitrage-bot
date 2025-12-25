# Project Context for Claude Sessions

## Server & Repo
- **Server:** DigitalOcean `167.172.24.118`
- **Repo:** `AustinHerbert/Polymarket-Kalshi-Arbitrage-bot`
- **Production Branch:** `claude/debug-arbitrage-bot-3zxvV`
- **Config:** `.env` file + private key at `/root/kalshi_private_key.pem`

---

## What I Mean When I Say...

| Term | What It Is | Where It Lives |
|------|-----------|----------------|
| **Arb Bot** | Main arbitrage bot - finds YES+NO < $1.00 across Kalshi/Polymarket (sports + crypto) | `src/main.rs`, `src/execution.rs`, `src/discovery.rs`, `src/crypto_discovery.rs` |
| **Dashboard** | Web UI on port 8080 showing trades, positions, analytics | `src/web_config.rs` (HTML embedded as `DASHBOARD_HTML` constant ~line 1150+) |
| **ML Optimizer** | Per-category threshold optimization (NFL, NBA, etc.) | `src/ml_optimizer.rs` |
| **Shared API** | Kalshi and Polymarket API clients | `src/kalshi.rs`, `src/polymarket.rs`, `src/polymarket_clob.rs` |

---

## Key Files

```
src/
├── main.rs              # Bot entry point
├── web_config.rs        # Dashboard server (port 8080) + embedded HTML
├── execution.rs         # Order execution engine
├── discovery.rs         # Sports market discovery
├── crypto_discovery.rs  # Crypto market discovery
├── ml_optimizer.rs      # ML threshold optimization
├── circuit_breaker.rs   # Risk management
├── position_tracker.rs  # Position & P&L tracking
├── trade_log.rs         # Trade persistence
├── insights.rs          # Analytics
├── metrics.rs           # Performance metrics
├── priority_queue.rs    # Opportunity prioritization
├── kalshi.rs            # Kalshi API client
├── polymarket.rs        # Polymarket WebSocket client
└── polymarket_clob.rs   # Polymarket order execution

dashboard_data/          # Runtime data
├── trades.json
├── positions.json
├── summary.json
└── ml_settings.json
```

---

## Dashboard API Endpoints

- `/` - Main dashboard UI
- `/api/config` - Bot configuration
- `/api/trades` - Trade history
- `/api/positions` - Open positions
- `/api/analytics` - Performance analytics

---

## Quick Commands

```bash
# SSH to server
ssh root@167.172.24.118

# Go to bot directory
cd /root/Polymarket-Kalshi-Arbitrage-bot

# Check bot status
ps aux | grep prediction-market

# View logs
tail -f bot.log

# Restart bot (dry run)
DRY_RUN=1 ./target/release/prediction-market-arbitrage

# Restart bot (live)
DRY_RUN=0 nohup ./target/release/prediction-market-arbitrage > bot.log 2>&1 &
```

---

## Critical Rules

1. **DO NOT BREAK THE PRODUCTION BOT** - It's live making real trades
2. **Always test with `DRY_RUN=1`** before any live changes
3. **Wait for confirmation** before proceeding to next steps

---

## Future Plans

Migration to modular multi-bot structure is planned. See `MIGRATION_PLAN.md` on branch `claude/document-bot-github-structure-TVgHq` for details.
