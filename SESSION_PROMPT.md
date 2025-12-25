# Universal Session Prompt for Polymarket Trading Platform

Copy everything below this line to start a new Claude session:

---

## Project Context

I have a prediction market arbitrage trading system running on DigitalOcean server `167.172.24.118`.

**Repo:** `AustinHerbert/Polymarket-Kalshi-Arbitrage-bot`
**Production Branch:** `claude/debug-arbitrage-bot-3zxvV`
**Config:** `.env` file + private key at `/root/kalshi_private_key.pem`

---

## System Components (What I Mean When I Say...)

| Term | What It Is | Location |
|------|-----------|----------|
| **Arb Bot** | The main arbitrage bot - detects and executes YES+NO < $1.00 opportunities across Kalshi and Polymarket (sports + crypto) | `src/main.rs`, `src/execution.rs`, `src/discovery.rs`, `src/crypto_discovery.rs` |
| **Dashboard** | Web UI on port 8080 showing trades, positions, analytics | `src/web_config.rs` (HTML embedded as `DASHBOARD_HTML` constant ~line 1150+) |
| **ML Optimizer** | Per-category threshold optimization (NFL, NBA, etc.) | `src/ml_optimizer.rs` |
| **Shared API** | Kalshi and Polymarket API clients | `src/kalshi.rs`, `src/polymarket.rs`, `src/polymarket_clob.rs` |

**Dashboard API Endpoints:**
- `/` - Main dashboard UI
- `/api/config` - Bot configuration
- `/api/trades` - Trade history
- `/api/positions` - Open positions
- `/api/analytics` - Performance analytics

**Key Files:**
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

## ⚠️ CRITICAL RULES

1. **DO NOT BREAK THE PRODUCTION BOT** - It's live and making real trades
2. **Always test with `DRY_RUN=1`** before any live changes
3. **Wait for my confirmation** before proceeding to next steps
4. **Don't rebuild until all issues are complete** - single build at the end

---

## Workflow Protocol (MANDATORY)

### 1. ANALYZE
Before writing any code:
- Explain what we're building/fixing in plain English
- Walk through the logic step-by-step
- Identify edge cases and potential failure points
- Get my approval before proceeding

### 2. TEST
- Run existing tests BEFORE making changes (establish baseline)
- Write new tests for any new functionality
- Run ALL tests AFTER changes to catch regressions
- Show me the test results

### 3. SYNC
After verifying ALL fixes work:
- Commit changes with a clear, descriptive message
- Push to the current GitHub branch immediately
- Confirm the push was successful

### 4. DEPLOY
Provide commands I can run to:
```bash
# On server (167.172.24.118):
cd /root/Polymarket-Kalshi-Arbitrage-bot
git pull origin [branch-name]
cargo build --release
# Then restart command
```

---

## Future Structure (Migration Planned)

We are planning to restructure into a modular multi-bot platform. See `MIGRATION_PLAN.md` on branch `claude/document-bot-github-structure-TVgHq` for details.

Target structure:
```
/polymarket-trading-platform/
├── /core/           # Shared API clients, config, utils
├── /bots/
│   ├── /arb-bot/    # Current production bot
│   ├── /spread-bot/ # Future spread farming bot
│   └── /[new-bot]/  # Template for new bots
├── /dashboard/      # Standalone dashboard for all bots
└── /data/           # Runtime data per bot
```

**Current work stays in the existing structure until migration is complete.**

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

# View dashboard
# Browser: http://167.172.24.118:8080
```

---

**I am not a coder.** Explain things in plain English. Confirm before making changes. Don't make assumptions - ask if unclear.

**What do you need help with today?**
