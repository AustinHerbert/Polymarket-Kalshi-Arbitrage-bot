# Prompt for polymarket-trading-platform repo

Copy everything below this line and paste into a Claude session connected to polymarket-trading-platform:

---

I need you to add documentation and Claude skill files to this repo.

## IMPORTANT RULES:
- DO NOT modify any existing .rs files
- DO NOT modify any Cargo.toml files
- ONLY create NEW markdown files
- These are documentation/helper files only - they won't affect running code

## CREATE THESE FILES:

### 1. Create file: ML_ARCHITECTURE.md (in repo root)

```markdown
# Multi-Bot ML Architecture

## ML Strategy Per Bot

Each bot uses the ML approach best suited to its strategy:

---

## ARB BOT - Bayesian Optimization + Multi-Armed Bandit

**Location:** `bots/arb-bot/src/ml_optimizer.rs`

**Why this ML:**
- Arb opportunities are binary (take it or don't)
- Needs to learn optimal thresholds per category
- Must adapt to changing market conditions

**What it learns:**
| Feature | What ML Optimizes |
|---------|-------------------|
| Threshold per league | "NFL needs 0.6% margin, NBA needs 0.4%" |
| Time-of-day patterns | "Crypto arbs at 3am have 90% fill rate" |
| Position sizing | "Max 50 contracts on EPL, 100 on NBA" |
| Fill rate prediction | "This opportunity has 75% chance of filling" |

**Key Files:**
- `bots/arb-bot/src/ml_optimizer.rs` - Bayesian threshold learning
- `bots/arb-bot/src/priority_queue.rs` - Opportunity prioritization
- `bots/arb-bot/src/insights.rs` - Analytics

---

## SPREAD BOT - Time Series + Gradient Boosting (Planned)

**Location:** `bots/spread-bot/src/`

**Why this ML:**
- Spreads move over time (need forecasting)
- Entry/exit timing is critical
- Classification problem: "Should I enter this spread?"

**What it will learn:**
| Feature | What ML Optimizes |
|---------|-------------------|
| Entry timing | "Enter when spread > X and volatility < Y" |
| Exit timing | "Exit after Z minutes or when profit hits target" |
| Risk assessment | "This trade has high/low risk profile" |

**Key Files:**
- `bots/spread-bot/src/market_maker.rs` - Core strategy
- `bots/spread-bot/src/config.rs` - Spread parameters

---

## SHARED INSIGHTS LAYER (Future)

**Location:** `core/src/insights/` (to be created)

**Purpose:**
- Cross-bot performance analysis
- Portfolio-level optimization
- Plain-English insight generation

---

## Dashboard ML Panel

**Location:** `bots/arb-bot/src/web_config.rs`

Dashboard shows per-bot ML insights:
- Current thresholds (auto-optimized)
- Performance by category
- ML recommendations
- Model confidence

---

## Adding ML to a New Bot

1. Create `ml/` folder in bot's `src/` directory
2. Implement optimizer trait from `core/`
3. Add to bot's main loop
4. Connect to dashboard for visualization

See `.claude/skills/add-new-bot.md` for full guide.
```

### 2. Create file: ARCHITECTURE.md (in repo root)

```markdown
# Polymarket Trading Platform Architecture

## Overview

```
polymarket-trading-platform/
│
├── core/                      # SHARED INFRASTRUCTURE
│   └── src/api/               # API clients (Polymarket, Kalshi)
│
├── bots/
│   ├── arb-bot/               # Arbitrage bot (production)
│   │   └── src/
│   │       ├── main.rs        # Entry point
│   │       ├── execution.rs   # Order execution
│   │       ├── discovery.rs   # Market matching
│   │       ├── ml_optimizer.rs # ML thresholds
│   │       └── web_config.rs  # Dashboard (port 8080)
│   │
│   └── spread-bot/            # Market making bot
│       └── src/
│           ├── main.rs        # Entry point
│           └── market_maker.rs # MM strategy
│
├── dashboard/                 # Placeholder (lives in arb-bot)
│
└── deploy/                    # Deployment scripts
    ├── deploy.sh              # Start/stop/logs
    └── *.service              # Systemd services
```

## Key Components

| Component | Location | Purpose |
|-----------|----------|---------|
| **Arb Bot** | `bots/arb-bot/` | YES+NO < $1.00 arbitrage |
| **Spread Bot** | `bots/spread-bot/` | Market making |
| **Dashboard** | `bots/arb-bot/src/web_config.rs` | Web UI (port 8080) |
| **Shared API** | `core/src/api/` | Polymarket/Kalshi clients |
| **Deploy** | `deploy/` | Service management |

## Server

- **IP:** 167.172.24.118
- **Dashboard:** http://167.172.24.118:8080
- **Bot Location:** /root/polymarket-trading-platform/

## Deployment

```bash
cd /root/polymarket-trading-platform
./deploy/deploy.sh arb-bot restart
./deploy/deploy.sh spread-bot restart
./deploy/deploy.sh arb-bot logs
```

## Adding a New Bot

See `.claude/skills/add-new-bot.md`
```

### 3. Create directory: .claude/skills/

Run: `mkdir -p .claude/skills`

### 4. Create file: .claude/skills/optimize-arb-bot.md

```markdown
# Skill: Optimize Arb Bot

## Trigger
User says: "optimize arb bot", "improve arb", "arb performance", "arb thresholds"

## Key Files
- `bots/arb-bot/src/ml_optimizer.rs` - ML threshold learning
- `bots/arb-bot/src/trade_log.rs` - Trade history
- `bots/arb-bot/src/insights.rs` - Analytics

## Steps
1. Read trade history and current ML settings
2. Analyze by category (NBA, NFL, crypto, EPL, etc.):
   - Fill rate per category
   - Profit per trade
   - Time-of-day patterns
3. Compare current thresholds vs optimal
4. Generate plain-English recommendations

## Output Format
"Arb Bot Analysis:

Performance by Category:
| Category | Trades | Fill Rate | Avg Profit | Threshold |
|----------|--------|-----------|------------|-----------|
| NBA      | 150    | 92%       | $1.23      | 0.45%     |

Recommendations:
1. [Specific actionable recommendation]
2. [Specific actionable recommendation]"
```

### 5. Create file: .claude/skills/optimize-spread-bot.md

```markdown
# Skill: Optimize Spread Bot

## Trigger
User says: "optimize spread bot", "improve spread", "spread performance", "market maker"

## Key Files
- `bots/spread-bot/src/market_maker.rs` - Core MM logic
- `bots/spread-bot/src/config.rs` - Spread parameters
- `bots/spread-bot/src/order_manager.rs` - Order handling

## Steps
1. Read spread bot trade history
2. Analyze:
   - Win rate by spread size
   - Hold duration patterns
   - Entry/exit timing
3. Compare to config settings
4. Generate recommendations

## Output Format
"Spread Bot Analysis:

Performance:
- Win Rate: X%
- Avg Hold: X minutes
- Best Spread Size: X%

Recommendations:
1. [Specific recommendation]
2. [Specific recommendation]"
```

### 6. Create file: .claude/skills/dashboard-update.md

```markdown
# Skill: Dashboard Update

## Trigger
User says: "update dashboard", "fix dashboard", "dashboard change", "add to dashboard"

## Key Files
- `bots/arb-bot/src/web_config.rs` - Dashboard server + embedded HTML
- HTML is in DASHBOARD_HTML constant (large string ~line 1000+)
- API endpoints defined in same file

## Dashboard Location
- Code: `bots/arb-bot/src/web_config.rs`
- URL: http://167.172.24.118:8080

## API Endpoints
- GET / - Main dashboard
- GET /api/config - Bot configuration
- GET /api/trades - Trade history
- GET /api/positions - Open positions
- GET /api/analytics - Performance analytics

## Steps
1. Read `bots/arb-bot/src/web_config.rs`
2. Find DASHBOARD_HTML constant
3. Make requested changes to HTML/CSS/JS
4. If adding new data, check/add API endpoint
5. Test compilation: `cargo build -p arb-bot`

## Caution
- Dashboard is embedded HTML - careful with quotes/escaping
- Always test build after changes
- Don't break existing functionality
```

### 7. Create file: .claude/skills/debug-bot.md

```markdown
# Skill: Debug Bot

## Trigger
User says: "debug", "bot not working", "error", "fix bug", "why isn't"

## Key Files by Issue Type

| Issue | Check These Files |
|-------|-------------------|
| No trades | `bots/arb-bot/src/discovery.rs`, `crypto_discovery.rs` |
| Fills failing | `bots/arb-bot/src/execution.rs` |
| Dashboard blank | `bots/arb-bot/src/web_config.rs` |
| Wrong prices | `bots/arb-bot/src/polymarket.rs`, `kalshi.rs` |
| Spread bot issues | `bots/spread-bot/src/market_maker.rs` |
| API errors | `core/src/api/` |

## Steps
1. Ask user to describe the problem
2. Check logs: `./deploy/deploy.sh [bot] logs`
3. Identify the component from table above
4. Search for error patterns
5. Explain issue in plain English
6. Propose minimal fix

## Log Commands
```bash
./deploy/deploy.sh arb-bot logs
./deploy/deploy.sh spread-bot logs
```
```

### 8. Create file: .claude/skills/deploy-changes.md

```markdown
# Skill: Deploy Changes

## Trigger
User says: "deploy", "push to server", "update server", "go live"

## Key Files
- `deploy/deploy.sh` - Service management script
- `deploy/arb-bot.service` - Arb bot systemd service
- `deploy/spread-bot.service` - Spread bot systemd service

## Pre-Deploy Checklist
- [ ] All changes tested
- [ ] No compiler errors: `cargo build --release`
- [ ] Changes committed to git
- [ ] Pushed to GitHub

## Deploy Commands

```bash
# On server (167.172.24.118):
cd /root/polymarket-trading-platform
git pull origin main
cargo build --release

# Restart specific bot:
./deploy/deploy.sh arb-bot restart
./deploy/deploy.sh spread-bot restart

# Check status:
./deploy/deploy.sh arb-bot status
./deploy/deploy.sh spread-bot status

# View logs:
./deploy/deploy.sh arb-bot logs
./deploy/deploy.sh spread-bot logs
```

## Rollback
```bash
git checkout HEAD~1 -- .
cargo build --release
./deploy/deploy.sh arb-bot restart
```
```

### 9. Create file: .claude/skills/analyze-trades.md

```markdown
# Skill: Analyze Trades

## Trigger
User says: "analyze trades", "trade history", "how am I doing", "performance"

## Key Files
- `bots/arb-bot/src/trade_log.rs` - Trade recording
- `bots/arb-bot/src/insights.rs` - Analytics

## Steps
1. Read trade history
2. Calculate:
   - Total trades, win rate, profit
   - By category breakdown
   - Time patterns
3. Generate insights

## Output Format
"Trade Analysis (Last 24h):

Summary:
- Total Trades: X
- Win Rate: X%
- Total Profit: $X

By Category:
| Category | Trades | Win Rate | Profit |
|----------|--------|----------|--------|
| NBA      | X      | X%       | $X     |

Insights:
1. [Observation]
2. [Recommendation]"
```

### 10. Create file: .claude/skills/add-new-bot.md

```markdown
# Skill: Add New Bot

## Trigger
User says: "add new bot", "create bot", "new trading bot"

## Steps

1. **Create folder structure:**
```bash
mkdir -p bots/{bot-name}/src
```

2. **Create Cargo.toml:**
```toml
[package]
name = "{bot-name}"
version = "0.1.0"
edition = "2021"

[dependencies]
trading-core = { path = "../../core" }
tokio = { workspace = true }
```

3. **Add to workspace** (root Cargo.toml):
```toml
members = [
    "core",
    "bots/arb-bot",
    "bots/spread-bot",
    "bots/{bot-name}",
    "dashboard",
]
```

4. **Create src/main.rs** with basic structure

5. **Create systemd service:**
```bash
cp deploy/arb-bot.service deploy/{bot-name}.service
# Edit to use new bot name
```

6. **Add to deploy.sh** if needed

## ML Selection Guide
| Strategy | Recommended ML |
|----------|----------------|
| Arbitrage | Bayesian + Bandit |
| Spread/MM | Time Series + XGBoost |
| Momentum | LSTM + RL |
```

### 11. Create file: .claude/skills/portfolio-insights.md

```markdown
# Skill: Portfolio Insights

## Trigger
User says: "portfolio", "overall performance", "all bots", "total profit"

## Steps
1. Gather data from all bots
2. Calculate:
   - Total P&L across all bots
   - Per-bot contribution
   - ROI comparison
3. Generate cross-bot insights

## Output Format
"Portfolio Summary:

Total Performance (Last 24h):
- Combined Profit: $X
- Total Trades: X

Per-Bot Breakdown:
| Bot | Profit | % of Total | ROI |
|-----|--------|------------|-----|
| Arb | $X     | X%         | X%  |
| Spread | $X  | X%         | X%  |

Insights:
1. [Which bot performing best]
2. [Capital allocation suggestion]

Recommendations:
- [ ] Action item"
```

### 12. Create file: .claude/skills/explain-ml-decision.md

```markdown
# Skill: Explain ML Decision

## Trigger
User says: "why did bot", "explain trade", "why this", "ML decision"

## Key Files
- `bots/arb-bot/src/ml_optimizer.rs` - ML logic
- `bots/arb-bot/src/trade_log.rs` - Trade records

## Steps
1. Find the specific trade in logs
2. Extract features at decision time
3. Explain in plain English

## Output Format
"Trade Explanation:

What happened:
- [Trade description]

Why the bot took this trade:
1. Price: [X] ✓
2. Category: [X] - fill rate [X%] ✓
3. Time: [X] - [good/bad] liquidity

Confidence: [X%]

What would have prevented this:
- If [condition]
- If [condition]"
```

### 13. After creating all files, commit and push:

```bash
git add ML_ARCHITECTURE.md ARCHITECTURE.md .claude/
git commit -m "Add ML architecture docs and Claude skills for project"
git push origin main
```

---

## SUMMARY OF FILES TO CREATE:

1. `ML_ARCHITECTURE.md` - ML strategy documentation
2. `ARCHITECTURE.md` - Project structure overview
3. `.claude/skills/optimize-arb-bot.md`
4. `.claude/skills/optimize-spread-bot.md`
5. `.claude/skills/dashboard-update.md`
6. `.claude/skills/debug-bot.md`
7. `.claude/skills/deploy-changes.md`
8. `.claude/skills/analyze-trades.md`
9. `.claude/skills/add-new-bot.md`
10. `.claude/skills/portfolio-insights.md`
11. `.claude/skills/explain-ml-decision.md`

These are ALL new files. No existing code is modified. Confirm when complete.
