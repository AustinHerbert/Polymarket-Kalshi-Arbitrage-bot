# MIGRATION PLAN: Modular Multi-Bot Trading Platform

---

## ⚠️ NUMBER ONE RULE: DO NOT BREAK ANYTHING ⚠️

**The arbitrage bot is LIVE IN PRODUCTION making real trades and real money.**

- **NEVER** stop, modify, or delete the current bot until new structure is 100% verified
- **ALWAYS** build new structure ALONGSIDE old (different folder)
- **ALWAYS** test with `DRY_RUN=1` before any live testing
- **ONLY** switch over after side-by-side verification proves new bot matches old bot behavior
- **KEEP** old bot as backup for 48+ hours after switchover

**If in doubt, DON'T DO IT. Ask first.**

---

## Current Production Bot (DO NOT TOUCH)

```
/root/Polymarket-Kalshi-Arbitrage-bot/    ← LEAVE THIS RUNNING
├── Branch: claude/debug-arbitrage-bot-3zxvV
├── Status: LIVE PRODUCTION
└── DO NOT MODIFY UNTIL NEW STRUCTURE VERIFIED
```

---

## Target Structure (Build in NEW location)

```
/root/polymarket-trading-platform/        ← BUILD HERE (separate folder)
│
├── /core/                                # Shared infrastructure
│   ├── /api/
│   │   ├── kalshi.rs                     # Kalshi REST + WebSocket
│   │   ├── polymarket.rs                 # Polymarket WebSocket
│   │   ├── polymarket_clob.rs            # Polymarket CLOB execution
│   │   └── mod.rs
│   │
│   ├── /config/
│   │   ├── mod.rs                        # Config loader
│   │   ├── leagues.rs                    # League configurations
│   │   └── .env                          # Credentials (gitignored)
│   │
│   ├── /utils/
│   │   ├── logging.rs                    # Shared logging
│   │   ├── types.rs                      # Common types
│   │   ├── cache.rs                      # Team code cache
│   │   └── mod.rs
│   │
│   └── /database/
│       ├── trade_log.rs                  # Trade persistence
│       ├── positions.rs                  # Position tracking
│       └── mod.rs
│
├── /bots/
│   ├── /arb-bot/                         # Arbitrage (Sports + Crypto)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── execution.rs
│   │   │   ├── discovery.rs
│   │   │   ├── crypto_discovery.rs
│   │   │   ├── circuit_breaker.rs
│   │   │   ├── ml_optimizer.rs
│   │   │   ├── priority_queue.rs
│   │   │   ├── priority_config.rs
│   │   │   ├── insights.rs
│   │   │   ├── metrics.rs
│   │   │   ├── balance_tracker.rs
│   │   │   └── opportunity_log.rs
│   │   └── README.md
│   │
│   ├── /spread-bot/                      # Spread farming (NEW)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   └── strategy.rs
│   │   └── README.md
│   │
│   └── /bot-template/                    # Template for future bots
│       ├── Cargo.toml.template
│       └── src/main.rs.template
│
├── /dashboard/                           # Unified dashboard
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── server.rs
│   │   ├── api.rs
│   │   └── html/
│   │       ├── index.html
│   │       ├── arb-bot.html
│   │       └── spread-bot.html
│   └── README.md
│
├── /data/                                # Runtime data (gitignored)
│   ├── arb-bot/
│   │   ├── trades.json
│   │   ├── positions.json
│   │   └── metrics.json
│   └── spread-bot/
│
├── Cargo.toml                            # Workspace root
├── ARCHITECTURE.md
├── docker-compose.yml
├── deploy.sh
└── .gitignore
```

---

## File Mapping: Current → New Location

### SHARED (goes to `/core/`)

| Current File | New Location |
|--------------|--------------|
| `kalshi.rs` | `core/api/kalshi.rs` |
| `polymarket.rs` | `core/api/polymarket.rs` |
| `polymarket_clob.rs` | `core/api/polymarket_clob.rs` |
| `config.rs` | `core/config/mod.rs` + `leagues.rs` |
| `cache.rs` | `core/utils/cache.rs` |
| `types.rs` (common parts) | `core/utils/types.rs` |
| `trade_log.rs` | `core/database/trade_log.rs` |
| `position_tracker.rs` | `core/database/positions.rs` |

### ARB BOT SPECIFIC (goes to `/bots/arb-bot/`)

| Current File | New Location |
|--------------|--------------|
| `main.rs` | `bots/arb-bot/src/main.rs` |
| `execution.rs` | `bots/arb-bot/src/execution.rs` |
| `discovery.rs` | `bots/arb-bot/src/discovery.rs` |
| `crypto_discovery.rs` | `bots/arb-bot/src/crypto_discovery.rs` |
| `circuit_breaker.rs` | `bots/arb-bot/src/circuit_breaker.rs` |
| `ml_optimizer.rs` | `bots/arb-bot/src/ml_optimizer.rs` |
| `priority_queue.rs` | `bots/arb-bot/src/priority_queue.rs` |
| `priority_config.rs` | `bots/arb-bot/src/priority_config.rs` |
| `insights.rs` | `bots/arb-bot/src/insights.rs` |
| `metrics.rs` | `bots/arb-bot/src/metrics.rs` |
| `balance_tracker.rs` | `bots/arb-bot/src/balance_tracker.rs` |
| `opportunity_log.rs` | `bots/arb-bot/src/opportunity_log.rs` |

### DASHBOARD (goes to `/dashboard/`)

| Current File | New Location |
|--------------|--------------|
| `web_config.rs` | `dashboard/src/server.rs` + `api.rs` |
| Embedded HTML | `dashboard/src/html/*.html` (extract) |

---

## Safe Migration Steps

### Phase 1: Setup New Structure (NO CHANGES TO OLD BOT)

```bash
# On server - create new folder NEXT TO old one
cd /root
mkdir polymarket-trading-platform
cd polymarket-trading-platform
git init

# Old bot stays at /root/Polymarket-Kalshi-Arbitrage-bot/ UNTOUCHED
```

1. Create folder structure
2. Create workspace `Cargo.toml`
3. **Copy** (not move) shared files to `core/`
4. Verify `core/` compiles: `cargo build -p trading-core`

**Checkpoint:** Old bot still running, new structure compiles separately

### Phase 2: Migrate Arb Bot (COPY, DON'T MOVE)

1. **Copy** arb-specific files to `bots/arb-bot/src/`
2. Update imports to use `trading-core`
3. Build: `cargo build -p arb-bot --release`
4. Test with DRY_RUN:
   ```bash
   DRY_RUN=1 ./target/release/arb-bot
   ```
5. **Compare output** to production bot logs

**Checkpoint:** Old bot STILL running. New bot tested in dry-run only.

### Phase 3: Side-by-Side Verification

1. Run new arb-bot in `DRY_RUN=1` mode
2. Compare detected opportunities with production bot
3. Verify:
   - Same markets discovered
   - Same prices received
   - Same arbitrage opportunities detected
   - Logs match expected behavior

**DO NOT PROCEED until new bot behavior matches old bot exactly**

### Phase 4: Dashboard Migration

1. **Copy** `web_config.rs` to `dashboard/src/`
2. Extract embedded HTML to separate files
3. Update API routes for multi-bot support
4. Test dashboard connects to **old bot's data** first
5. Then test with new bot's data

**Checkpoint:** Dashboard works with both old and new data sources

### Phase 5: Cutover (ONLY AFTER FULL VERIFICATION)

```bash
# 1. Stop old bot
cd /root/Polymarket-Kalshi-Arbitrage-bot
pkill -f "target/release/prediction-market"

# 2. Start new bot (LIVE)
cd /root/polymarket-trading-platform
DRY_RUN=0 ./target/release/arb-bot &

# 3. Monitor closely for 1 hour
tail -f data/arb-bot/logs/bot.log

# 4. If problems, IMMEDIATELY roll back:
pkill -f "target/release/arb-bot"
cd /root/Polymarket-Kalshi-Arbitrage-bot
DRY_RUN=0 ./target/release/prediction-market-arbitrage &
```

### Phase 6: Cleanup (48+ HOURS AFTER SUCCESSFUL CUTOVER)

1. Verify new bot has been running stable for 48+ hours
2. Verify trades executing correctly
3. Verify dashboard showing correct data
4. **Only then** archive old folder:
   ```bash
   mv /root/Polymarket-Kalshi-Arbitrage-bot /root/old-bot-backup
   ```
5. Keep backup for another week before deleting

---

## Cargo Workspace Setup

### Root `Cargo.toml`
```toml
[workspace]
resolver = "2"
members = [
    "core",
    "bots/arb-bot",
    "bots/spread-bot",
    "dashboard",
]

[workspace.dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
reqwest = { version = "0.11", features = ["json"] }
tokio-tungstenite = { version = "0.21", features = ["native-tls"] }
ethers = { version = "2.0", features = ["legacy"] }
```

### `core/Cargo.toml`
```toml
[package]
name = "trading-core"
version = "1.0.0"
edition = "2021"

[dependencies]
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
tracing.workspace = true
reqwest.workspace = true
tokio-tungstenite.workspace = true
ethers.workspace = true
# ... other shared deps
```

### `bots/arb-bot/Cargo.toml`
```toml
[package]
name = "arb-bot"
version = "1.0.0"
edition = "2021"

[dependencies]
trading-core = { path = "../../core" }
tokio.workspace = true
serde.workspace = true
anyhow.workspace = true
tracing.workspace = true
# ... arb-specific deps
```

---

## How Bots Connect to Dashboard

```
┌─────────────────┐
│  OLD BOT        │ ← KEEP RUNNING until cutover
│  (production)   │
└─────────────────┘

┌─────────────────┐     writes      ┌──────────────────┐
│  NEW ARB BOT    │ ───────────────►│  /data/arb-bot/  │
│  (test first)   │                 │  trades.json     │
└─────────────────┘                 │  positions.json  │
                                    └────────┬─────────┘
┌─────────────────┐     writes               │
│  SPREAD BOT     │ ───────────────►┌────────┴─────────┐
│  (new)          │                 │ /data/spread-bot/│
└─────────────────┘                 └────────┬─────────┘
                                             │ reads
                                             ▼
                                    ┌─────────────────┐
                                    │    DASHBOARD    │
                                    │   (port 8080)   │
                                    └─────────────────┘
```

---

## How to Add a New Bot (Future)

1. **Copy template:**
   ```bash
   cp -r bots/bot-template bots/my-new-bot
   ```

2. **Update `Cargo.toml`:**
   ```toml
   [package]
   name = "my-new-bot"

   [dependencies]
   trading-core = { path = "../../core" }
   ```

3. **Add to workspace** (root `Cargo.toml`):
   ```toml
   members = [
       "core",
       "bots/arb-bot",
       "bots/spread-bot",
       "bots/my-new-bot",  # Add this
       "dashboard",
   ]
   ```

4. **Create data folder:**
   ```bash
   mkdir -p data/my-new-bot
   ```

5. **Add dashboard page:**
   - Create `dashboard/src/html/my-new-bot.html`
   - Add route in `dashboard/src/api.rs`

6. **Implement bot logic** in `bots/my-new-bot/src/`

7. **Test with DRY_RUN=1** before any live trading

---

## Deployment Workflow

```bash
# Development (push changes):
git add .
git commit -m "Update"
git push origin main

# Server (pull and rebuild):
ssh root@167.172.24.118
cd /root/polymarket-trading-platform
git pull origin main
cargo build --release

# Restart specific bot:
./deploy.sh arb-bot restart

# Check status:
./deploy.sh arb-bot status
```

### `deploy.sh`
```bash
#!/bin/bash
BOT=$1
ACTION=$2

case $ACTION in
  restart)
    echo "Stopping $BOT..."
    pkill -f "target/release/$BOT" || true
    sleep 2
    echo "Starting $BOT..."
    nohup ./target/release/$BOT > data/$BOT/logs/bot.log 2>&1 &
    echo "$BOT restarted (PID: $!)"
    ;;
  stop)
    pkill -f "target/release/$BOT"
    echo "$BOT stopped"
    ;;
  status)
    if pgrep -f "target/release/$BOT" > /dev/null; then
      echo "$BOT is RUNNING"
    else
      echo "$BOT is STOPPED"
    fi
    ;;
esac
```

---

## Rollback Plan

If ANYTHING goes wrong after cutover:

```bash
# 1. Stop new bot immediately
pkill -f "target/release/arb-bot"

# 2. Start old bot
cd /root/Polymarket-Kalshi-Arbitrage-bot
DRY_RUN=0 ./target/release/prediction-market-arbitrage &

# 3. Verify old bot is running
ps aux | grep prediction-market

# 4. Investigate what went wrong before trying again
```

---

## Checklist Before Cutover

- [ ] New `core/` compiles independently
- [ ] New `arb-bot` compiles and links to core
- [ ] New `arb-bot` runs in DRY_RUN mode without errors
- [ ] Market discovery finds same markets as old bot
- [ ] WebSocket connections work (Kalshi + Polymarket)
- [ ] Arbitrage detection matches old bot output
- [ ] Dashboard displays data correctly
- [ ] Tested for at least 1 hour in DRY_RUN
- [ ] Rollback plan is ready
- [ ] Old bot backup location confirmed

**DO NOT CUTOVER UNTIL ALL BOXES CHECKED**

---

## Summary

1. **Build new structure in `/root/polymarket-trading-platform/`** (separate from old)
2. **Old bot keeps running** at `/root/Polymarket-Kalshi-Arbitrage-bot/`
3. **Test new bot thoroughly** in DRY_RUN mode
4. **Side-by-side verification** before any cutover
5. **Quick rollback plan** ready if anything fails
6. **Keep old bot as backup** for 48+ hours after cutover
