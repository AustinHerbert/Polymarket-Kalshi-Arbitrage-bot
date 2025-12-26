# INSTRUCTIONS: Add Documentation and Skills to polymarket-trading-platform

## WHAT TO DO

Create these 11 files. Do not modify any existing files.

---

## FILE 1: ML_ARCHITECTURE.md

Create in repo root:

# Multi-Bot ML Architecture

## ML Strategy Per Bot

| Bot | ML Approach | Why |
|-----|-------------|-----|
| **Arb Bot** | Bayesian Optimization + Multi-Armed Bandit | Fast threshold tuning per category |
| **Spread Bot** | Time Series + XGBoost | Entry/exit timing prediction |

### Arb Bot ML

**Location:** `bots/arb-bot/src/ml_optimizer.rs`

**What it learns:**
- Threshold per league (NFL needs 0.6%, NBA needs 0.4%)
- Time-of-day patterns (crypto at 3am = 90% fill rate)
- Position sizing per category
- Fill rate prediction

**Key Files:**
- `bots/arb-bot/src/ml_optimizer.rs`
- `bots/arb-bot/src/priority_queue.rs`
- `bots/arb-bot/src/insights.rs`

### Spread Bot ML (Planned)

**Location:** `bots/spread-bot/src/`

**What it will learn:**
- Entry timing
- Exit timing
- Risk assessment

**Key Files:**
- `bots/spread-bot/src/market_maker.rs`
- `bots/spread-bot/src/config.rs`

### Dashboard ML Panel

**Location:** `bots/arb-bot/src/web_config.rs`

Shows: thresholds, performance by category, ML recommendations, confidence

---

## FILE 2: ARCHITECTURE.md

Create in repo root:

# Polymarket Trading Platform Architecture

## Structure

```
polymarket-trading-platform/
├── core/                      # Shared API clients
│   └── src/api/
├── bots/
│   ├── arb-bot/               # Arbitrage bot
│   │   └── src/
│   │       ├── main.rs
│   │       ├── execution.rs
│   │       ├── ml_optimizer.rs
│   │       └── web_config.rs  # Dashboard
│   └── spread-bot/            # Market making
│       └── src/
│           ├── main.rs
│           └── market_maker.rs
├── dashboard/                 # Placeholder
└── deploy/                    # Deployment scripts
    └── deploy.sh
```

## Key Info

| Component | Location |
|-----------|----------|
| Arb Bot | `bots/arb-bot/` |
| Spread Bot | `bots/spread-bot/` |
| Dashboard | `bots/arb-bot/src/web_config.rs` (port 8080) |
| Deploy | `deploy/deploy.sh` |

## Server

- IP: 167.172.24.118
- Dashboard: http://167.172.24.118:8080

## Deploy Commands

```bash
cd /root/polymarket-trading-platform
./deploy/deploy.sh arb-bot restart
./deploy/deploy.sh arb-bot logs
```

---

## FILE 3: .claude/skills/optimize-arb-bot.md

Create directory `.claude/skills/` first, then create this file:

# Skill: Optimize Arb Bot

## Trigger
"optimize arb bot", "improve arb", "arb thresholds"

## Key Files
- `bots/arb-bot/src/ml_optimizer.rs`
- `bots/arb-bot/src/trade_log.rs`

## Steps
1. Read trade history
2. Analyze by category (fill rate, profit, time patterns)
3. Generate recommendations

---

## FILE 4: .claude/skills/optimize-spread-bot.md

# Skill: Optimize Spread Bot

## Trigger
"optimize spread bot", "spread performance"

## Key Files
- `bots/spread-bot/src/market_maker.rs`
- `bots/spread-bot/src/config.rs`

## Steps
1. Read spread bot trades
2. Analyze win rate, hold duration, entry/exit
3. Generate recommendations

---

## FILE 5: .claude/skills/dashboard-update.md

# Skill: Dashboard Update

## Trigger
"update dashboard", "fix dashboard"

## Key Files
- `bots/arb-bot/src/web_config.rs` - HTML in DASHBOARD_HTML constant (~line 1000+)

## API Endpoints
- GET / - Main dashboard
- GET /api/trades - Trade history
- GET /api/positions - Positions
- GET /api/analytics - Analytics

## Steps
1. Read `bots/arb-bot/src/web_config.rs`
2. Find DASHBOARD_HTML
3. Make changes
4. Test: `cargo build -p arb-bot`

---

## FILE 6: .claude/skills/debug-bot.md

# Skill: Debug Bot

## Trigger
"debug", "error", "not working"

## Key Files by Issue
| Issue | File |
|-------|------|
| No trades | `bots/arb-bot/src/discovery.rs` |
| Fills failing | `bots/arb-bot/src/execution.rs` |
| Dashboard blank | `bots/arb-bot/src/web_config.rs` |
| API errors | `core/src/api/` |

## Logs
```bash
./deploy/deploy.sh arb-bot logs
./deploy/deploy.sh spread-bot logs
```

---

## FILE 7: .claude/skills/deploy-changes.md

# Skill: Deploy Changes

## Trigger
"deploy", "push to server"

## Commands
```bash
# On server 167.172.24.118:
cd /root/polymarket-trading-platform
git pull origin main
cargo build --release
./deploy/deploy.sh arb-bot restart
./deploy/deploy.sh spread-bot restart
```

## Rollback
```bash
git checkout HEAD~1 -- .
cargo build --release
./deploy/deploy.sh arb-bot restart
```

---

## FILE 8: .claude/skills/analyze-trades.md

# Skill: Analyze Trades

## Trigger
"analyze trades", "performance", "how am I doing"

## Key Files
- `bots/arb-bot/src/trade_log.rs`
- `bots/arb-bot/src/insights.rs`

## Steps
1. Read trade history
2. Calculate: total trades, win rate, profit by category
3. Generate insights

---

## FILE 9: .claude/skills/add-new-bot.md

# Skill: Add New Bot

## Trigger
"add new bot", "create bot"

## Steps
1. `mkdir -p bots/{name}/src`
2. Create Cargo.toml with `trading-core = { path = "../../core" }`
3. Add to root Cargo.toml workspace members
4. Create src/main.rs
5. Copy and edit deploy/{name}.service

## ML by Strategy
| Strategy | ML |
|----------|-----|
| Arbitrage | Bayesian |
| Spread | XGBoost |
| Momentum | LSTM |

---

## FILE 10: .claude/skills/portfolio-insights.md

# Skill: Portfolio Insights

## Trigger
"portfolio", "all bots", "total profit"

## Steps
1. Gather data from all bots
2. Calculate total P&L, per-bot ROI
3. Generate cross-bot insights

---

## FILE 11: .claude/skills/explain-ml-decision.md

# Skill: Explain ML Decision

## Trigger
"why did bot", "explain trade"

## Key Files
- `bots/arb-bot/src/ml_optimizer.rs`
- `bots/arb-bot/src/trade_log.rs`

## Steps
1. Find the trade in logs
2. Extract features at decision time
3. Explain in plain English

---

## AFTER CREATING ALL FILES

```bash
git add ML_ARCHITECTURE.md ARCHITECTURE.md .claude/
git commit -m "Add ML architecture docs and Claude skills"
git push origin main
```

---

## CHECKLIST

- [ ] ML_ARCHITECTURE.md created
- [ ] ARCHITECTURE.md created
- [ ] .claude/skills/ directory created
- [ ] optimize-arb-bot.md created
- [ ] optimize-spread-bot.md created
- [ ] dashboard-update.md created
- [ ] debug-bot.md created
- [ ] deploy-changes.md created
- [ ] analyze-trades.md created
- [ ] add-new-bot.md created
- [ ] portfolio-insights.md created
- [ ] explain-ml-decision.md created
- [ ] All files committed and pushed
