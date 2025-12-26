# Claude Session Context - Polymarket/Kalshi Trading System

**Server:** DigitalOcean 167.172.24.118
**Repo:** `Polymarket-Kalshi-Arbitrage-bot`

---

## ASK AT SESSION START

When starting a new session, ask the user:

> "Which component are you working on today?"
> 1. **Arb Bot** - Core arbitrage detection & execution
> 2. **Spread Bot** - Spread farming with ML optimization + Dashboard
> 3. **Dashboard** - Web UI (embedded in spread bot)
> 4. **All/Infrastructure** - Cross-cutting changes

---

## System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    TRADING SYSTEM ARCHITECTURE                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────┐       ┌─────────────────────┐         │
│  │      ARB BOT        │       │     SPREAD BOT      │         │
│  │  (Base Arbitrage)   │       │ (ML + Dashboard)    │         │
│  │                     │       │                     │         │
│  │  • Cross-platform   │       │  • All arb features │         │
│  │  • Kalshi-Poly      │       │  • ML optimizer     │         │
│  │  • Poly-Poly        │       │  • Trade logging    │         │
│  │  • Kalshi-Kalshi    │       │  • Web dashboard    │         │
│  │  • Circuit breaker  │       │  • Per-category     │         │
│  │  • Position tracker │       │    thresholds       │         │
│  └─────────────────────┘       └──────────┬──────────┘         │
│                                           │                     │
│                                           ▼                     │
│                                ┌─────────────────────┐         │
│                                │     DASHBOARD       │         │
│                                │   (Port 8080)       │         │
│                                │                     │         │
│                                │  • /api/config      │         │
│                                │  • /api/trades      │         │
│                                │  • /api/positions   │         │
│                                │  • /api/analytics   │         │
│                                └─────────────────────┘         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 1. ARB BOT (Core Arbitrage)

**Branch:** `main` or base branch
**Purpose:** Detects and executes arbitrage opportunities across prediction markets

### Directory Structure
```
src/
├── main.rs              # Entry point, WebSocket orchestration
├── types.rs             # Core types (MarketPair, AtomicOrderbook, GlobalState)
├── execution.rs         # Concurrent order execution engine
├── discovery.rs         # Market matching (Kalshi ↔ Polymarket)
├── circuit_breaker.rs   # Risk management (position limits, daily loss)
├── position_tracker.rs  # P&L tracking, cost basis
├── cache.rs             # Team code mapping cache
├── config.rs            # League configs, thresholds
├── kalshi.rs            # Kalshi REST + WebSocket client
├── polymarket.rs        # Polymarket WebSocket client
└── polymarket_clob.rs   # Polymarket CLOB order execution
```

### Arbitrage Types
| Type | Description |
|------|-------------|
| `poly_yes_kalshi_no` | Buy YES on Polymarket + NO on Kalshi |
| `kalshi_yes_poly_no` | Buy YES on Kalshi + NO on Polymarket |
| `poly_poly` | Buy YES + NO both on Polymarket |
| `kalshi_kalshi` | Buy YES + NO both on Kalshi |

### Key Files
- **types.rs** (1,200+ lines) - Lock-free atomic orderbook, packed 64-bit state
- **execution.rs** - Order placement, deduplication, exposure management
- **discovery.rs** - Market matching with 2-hour cache TTL
- **circuit_breaker.rs** - Max position per market, total exposure, daily loss limits

### Environment Variables
```bash
KALSHI_API_KEY_ID=...
KALSHI_PRIVATE_KEY_PATH=/path/to/key.pem
POLY_PRIVATE_KEY=0x...
POLY_FUNDER=0x...
DRY_RUN=1              # Set to 0 for live trading
RUST_LOG=info
CB_ENABLED=true
CB_MAX_POSITION_PER_MARKET=100
CB_MAX_TOTAL_POSITION=500
CB_MAX_DAILY_LOSS=5000
```

---

## 2. SPREAD BOT (ML Optimization + Dashboard)

**Branch:** `claude/spread-farming-strategy-Vq012` (or deployed version)
**Purpose:** Enhanced arbitrage with ML per-category optimization and web dashboard
**Location on Server:** `/root/Polymarket-Kalshi-Arbitrage-bot/`

### Directory Structure
```
src/
├── main.rs              # Bot entry point
├── web_config.rs        # Dashboard server (Axum, port 8080)
│                        # Contains DASHBOARD_HTML constant (~line 1150+)
├── trade_log.rs         # Trade persistence to JSON
├── ml_optimizer.rs      # ML per-category threshold optimization
├── opportunity_log.rs   # Opportunity logging with category normalization
├── execution.rs         # Trade execution engine
├── position_tracker.rs  # Open position management
└── [all arb bot files]  # Inherits full arb bot functionality

dashboard_data/          # Runtime data directory
├── trades.json          # All trade records
├── summary.json         # Trade statistics
├── ml_settings.json     # ML optimizer settings per category
└── open_positions.json  # Current open positions
```

### Additional Modules (Beyond Arb Bot)
| File | Purpose |
|------|---------|
| `web_config.rs` | Axum web server, REST API, embedded dashboard HTML |
| `trade_log.rs` | JSON persistence for trade history |
| `ml_optimizer.rs` | Per-category threshold learning (NFL, NBA, MLB, NHL, NCAA, Soccer, Crypto) |
| `opportunity_log.rs` | Logs opportunities with normalized categories |

### Dashboard API Endpoints
| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Serves embedded HTML dashboard |
| `/api/config` | GET | Current bot configuration |
| `/api/trades` | GET | Trade history from trades.json |
| `/api/positions` | GET | Open positions |
| `/api/analytics` | GET | Performance analytics |

### Startup
```bash
./run_bot.sh
# or
cargo run --release
```

---

## 3. DASHBOARD (Embedded in Spread Bot)

**Location:** `src/web_config.rs` (line ~1150+ as `DASHBOARD_HTML` constant)
**Port:** 8080
**Tech:** Axum (Rust) backend, vanilla HTML/JS frontend

### Key Features
- Real-time trade display
- Position monitoring
- Per-category performance analytics
- ML threshold visualization

### To Modify Dashboard
1. Find `DASHBOARD_HTML` constant in `src/web_config.rs`
2. Edit the embedded HTML/CSS/JS
3. Rebuild and restart bot

---

## 4. CONNECTIONS & DATA FLOW

```
┌──────────────┐    WebSocket    ┌──────────────┐
│   Kalshi     │◄───────────────►│              │
│   Exchange   │                 │              │
└──────────────┘                 │              │
                                 │   SPREAD     │────► dashboard_data/
┌──────────────┐    WebSocket    │     BOT      │      ├── trades.json
│  Polymarket  │◄───────────────►│              │      ├── summary.json
│   Exchange   │                 │              │      ├── ml_settings.json
└──────────────┘                 │              │      └── open_positions.json
                                 └──────┬───────┘
                                        │
                                        ▼ Port 8080
                                 ┌──────────────┐
                                 │  Dashboard   │
                                 │  (Browser)   │
                                 └──────────────┘
```

---

## 5. COMMON TASKS

### Arb Bot
- Modify arbitrage thresholds: `src/config.rs` → `ARB_THRESHOLD`
- Add new leagues: `src/config.rs` → `ENABLED_LEAGUES`
- Change circuit breaker limits: Environment variables or `src/circuit_breaker.rs`

### Spread Bot
- Modify ML thresholds: `src/ml_optimizer.rs`
- Change dashboard UI: `src/web_config.rs` → `DASHBOARD_HTML`
- Adjust trade logging: `src/trade_log.rs`

### Dashboard
- Add new API endpoint: `src/web_config.rs` (router configuration)
- Modify UI: `src/web_config.rs` → `DASHBOARD_HTML` constant
- Change data format: Corresponding `dashboard_data/*.json` consumers

---

## 6. BRANCHES

| Branch | Purpose |
|--------|---------|
| `main` / initial commit | Base arb bot |
| `claude/spread-farming-strategy-*` | Spread bot with ML + Dashboard |
| `claude/document-bot-github-structure-*` | Documentation updates |

---

## 7. SERVER ACCESS

```bash
ssh root@167.172.24.118
cd /root/Polymarket-Kalshi-Arbitrage-bot
```

---

## Quick Reference

| Component | Key File | Port | Branch |
|-----------|----------|------|--------|
| Arb Bot | `src/main.rs` | - | main |
| Spread Bot | `src/main.rs` + `src/web_config.rs` | 8080 | spread-farming-* |
| Dashboard | `src/web_config.rs:DASHBOARD_HTML` | 8080 | (same as spread) |
| ML Optimizer | `src/ml_optimizer.rs` | - | spread-farming-* |
