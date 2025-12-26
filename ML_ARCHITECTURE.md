# Multi-Bot ML Architecture & Skills

## ML Strategy Per Bot

Each bot gets the ML approach that fits its strategy best:

---

## 1. ARB BOT - Reinforcement Learning + Bayesian Optimization

**Why this ML:**
- Arb opportunities are binary (take it or don't)
- Needs to learn optimal thresholds per category
- Must adapt to changing market conditions
- Fast decisions required

**What it learns:**
| Feature | What ML Optimizes |
|---------|-------------------|
| Threshold per league | "NFL needs 0.6% margin, NBA needs 0.4%" |
| Time-of-day patterns | "Crypto arbs at 3am have 90% fill rate vs 60% daytime" |
| Position sizing | "Max 50 contracts on EPL, 100 on NBA" |
| Fill rate prediction | "This opportunity has 75% chance of filling both legs" |

**Implementation:**
```
/bots/arb-bot/src/
├── ml/
│   ├── mod.rs
│   ├── threshold_optimizer.rs    # Bayesian optimization for thresholds
│   ├── fill_predictor.rs         # Predicts if both legs will fill
│   ├── reward_calculator.rs      # Calculates reward signal for RL
│   └── feature_extractor.rs      # Extracts features from market data
```

**Algorithm:**
- Thompson Sampling (Bayesian) for threshold optimization
- Simple neural net or XGBoost for fill prediction
- Multi-armed bandit for category selection

---

## 2. SPREAD BOT - Time Series + Gradient Boosting

**Why this ML:**
- Spreads move over time (need forecasting)
- Entry/exit timing is critical
- More features to consider (volatility, volume, time to event)
- Classification problem: "Should I enter this spread?"

**What it learns:**
| Feature | What ML Optimizes |
|---------|-------------------|
| Entry timing | "Enter when spread > X and volatility < Y" |
| Exit timing | "Exit after Z minutes or when profit hits target" |
| Spread direction | "This spread will likely widen/narrow" |
| Risk assessment | "This trade has high/low risk profile" |

**Implementation:**
```
/bots/spread-bot/src/
├── ml/
│   ├── mod.rs
│   ├── spread_predictor.rs       # LSTM or Prophet for spread forecasting
│   ├── entry_classifier.rs       # XGBoost: should I enter?
│   ├── exit_optimizer.rs         # RL for optimal exit timing
│   └── risk_scorer.rs            # Risk assessment model
```

**Algorithm:**
- XGBoost/LightGBM for entry classification
- LSTM or Facebook Prophet for spread forecasting
- Q-Learning for exit optimization

---

## 3. FUTURE BOTS - Template with Pluggable ML

**Implementation:**
```
/bots/bot-template/src/
├── ml/
│   ├── mod.rs
│   ├── base_optimizer.rs         # Interface all ML must implement
│   └── examples/
│       ├── bayesian.rs           # Bayesian optimization example
│       ├── reinforcement.rs      # RL example
│       └── gradient_boost.rs     # XGBoost example
```

---

## 4. SHARED INSIGHTS LAYER - Ensemble + LLM

**Why this approach:**
- Combines data from all bots
- Generates human-readable insights
- Portfolio-level optimization

**What it does:**
| Function | How |
|----------|-----|
| Performance ranking | Compare ROI across bots |
| Capital allocation | Suggest where to put more money |
| Anomaly detection | "Arb bot performance dropped 50%" |
| Plain English insights | LLM generates explanations |

**Implementation:**
```
/core/insights/
├── mod.rs
├── portfolio_analyzer.rs         # Cross-bot performance
├── capital_allocator.rs          # Where to deploy capital
├── anomaly_detector.rs           # Isolation Forest for anomalies
└── insight_generator.rs          # LLM-powered explanations
```

---

## Skills for Claude Code

Create these in `.claude/skills/`:

### Skill 1: `optimize-arb-bot`
```markdown
# Skill: Optimize Arb Bot

## When to Use
User says: "optimize arb bot", "improve arb thresholds", "arb bot performance"

## What to Do
1. Read `data/arb-bot/trades.json` and `data/arb-bot/metrics.json`
2. Analyze performance by category (NFL, NBA, crypto, etc.)
3. Check current thresholds in `ml_settings.json`
4. Calculate optimal thresholds based on:
   - Fill rate per category
   - Profit per category
   - Time-of-day patterns
5. Generate plain-English recommendations

## Output Format
"Arb Bot Analysis:
- Best performing: NBA (0.8% avg profit, 92% fill rate)
- Worst performing: NFL (0.3% avg profit, 65% fill rate)

Recommendations:
1. Increase NBA position size from 50 to 75 contracts
2. Raise NFL threshold from 0.5% to 0.7% (too many failed fills)
3. Focus crypto trading between 2-6am UTC (best fill rates)"
```

### Skill 2: `optimize-spread-bot`
```markdown
# Skill: Optimize Spread Bot

## When to Use
User says: "optimize spread bot", "improve spread strategy", "spread bot performance"

## What to Do
1. Read `data/spread-bot/trades.json` and `data/spread-bot/metrics.json`
2. Analyze entry/exit patterns
3. Check win rate by:
   - Hold duration
   - Entry spread size
   - Market volatility at entry
4. Generate recommendations

## Output Format
"Spread Bot Analysis:
- Win rate: 67% (target: 70%)
- Avg hold time: 23 minutes
- Best entries: When spread > 2.5% and volatility < 0.3

Recommendations:
1. Reduce max hold time from 60 to 30 minutes (profits decay after 25min)
2. Only enter when spread > 2.5% (current 2.0% has 55% win rate)
3. Add volatility filter: skip when VIX > 25"
```

### Skill 3: `portfolio-insights`
```markdown
# Skill: Portfolio Insights

## When to Use
User says: "portfolio insights", "overall performance", "how are my bots doing"

## What to Do
1. Read data from ALL bot folders in `/data/`
2. Calculate:
   - Total P&L across all bots
   - ROI per bot
   - Capital allocation efficiency
3. Identify:
   - Best performing bot
   - Underperforming bots
   - Anomalies or issues
4. Generate portfolio-level recommendations

## Output Format
"Portfolio Summary (Last 24h):
- Total Profit: $150
- Arb Bot: $120 (80% of profits, using 60% of capital)
- Spread Bot: $30 (20% of profits, using 40% of capital)

Insights:
1. Arb Bot has 2x better capital efficiency
2. Consider reallocating 10% capital from Spread to Arb
3. Spread Bot win rate dropped 15% - investigate

Action Items:
- [ ] Review Spread Bot entry criteria
- [ ] Increase Arb Bot position limits"
```

### Skill 4: `add-new-bot`
```markdown
# Skill: Add New Bot

## When to Use
User says: "add new bot", "create bot", "new trading bot"

## What to Do
1. Ask user for:
   - Bot name
   - Strategy type (arbitrage, spread, market making, etc.)
   - Markets to trade
2. Create folder structure from template
3. Select appropriate ML approach based on strategy
4. Set up data logging
5. Add to dashboard

## Steps
1. Copy `/bots/bot-template/` to `/bots/{new-bot-name}/`
2. Update `Cargo.toml` with bot name
3. Add to workspace in root `Cargo.toml`
4. Create `/data/{new-bot-name}/` folder
5. Add dashboard page
6. Configure ML based on strategy type

## ML Selection Guide
| Strategy Type | Recommended ML |
|--------------|----------------|
| Arbitrage | Bayesian + Bandit |
| Spread/Mean Reversion | Time Series + XGBoost |
| Momentum | LSTM + RL |
| Market Making | Deep RL (PPO/SAC) |
```

### Skill 5: `explain-ml-decision`
```markdown
# Skill: Explain ML Decision

## When to Use
User says: "why did bot do X", "explain this trade", "why this threshold"

## What to Do
1. Find the specific trade/decision in logs
2. Extract the ML features at decision time
3. Explain in plain English:
   - What features the ML saw
   - Why it made that decision
   - Confidence level
   - What would change the decision

## Output Format
"Trade Explanation:

The bot took this arb opportunity because:
1. Combined price: 97.5¢ (threshold: 99.5¢) ✓
2. Category: NBA - historically 89% fill rate ✓
3. Time: 7pm EST - peak liquidity hours ✓
4. Size available: 150 contracts (wanted 100) ✓

Confidence: 85%

What would have stopped this trade:
- Combined price > 99.5¢
- Fill rate prediction < 60%
- Already at max position for this market"
```

---

## File Structure with ML

```
/polymarket-trading-platform/
│
├── /core/
│   ├── /api/                      # Shared API clients
│   ├── /ml-common/                # Shared ML utilities
│   │   ├── feature_store.rs       # Common feature extraction
│   │   ├── model_persistence.rs   # Save/load models
│   │   └── metrics.rs             # ML performance metrics
│   └── /insights/                 # Cross-bot insights
│       ├── portfolio_analyzer.rs
│       ├── capital_allocator.rs
│       └── insight_generator.rs   # LLM-powered explanations
│
├── /bots/
│   ├── /arb-bot/
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── execution.rs
│   │   │   └── ml/
│   │   │       ├── mod.rs
│   │   │       ├── threshold_optimizer.rs  # Bayesian
│   │   │       ├── fill_predictor.rs       # XGBoost
│   │   │       └── category_selector.rs    # Multi-armed bandit
│   │   └── models/                # Saved ML models
│   │       ├── threshold_model.bin
│   │       └── fill_model.bin
│   │
│   ├── /spread-bot/
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── strategy.rs
│   │   │   └── ml/
│   │   │       ├── mod.rs
│   │   │       ├── spread_predictor.rs     # LSTM
│   │   │       ├── entry_classifier.rs     # XGBoost
│   │   │       └── exit_optimizer.rs       # Q-Learning
│   │   └── models/
│   │       ├── spread_lstm.bin
│   │       └── entry_model.bin
│   │
│   └── /bot-template/
│       └── src/ml/
│           └── base_optimizer.rs   # Interface to implement
│
├── /dashboard/
│   ├── src/
│   │   ├── pages/
│   │   │   ├── arb_bot.rs         # Arb bot view + ML insights
│   │   │   ├── spread_bot.rs      # Spread bot view + ML insights
│   │   │   └── portfolio.rs       # Cross-bot insights
│   │   └── components/
│   │       ├── ml_insights.rs     # ML explanation component
│   │       └── recommendations.rs # Action items component
│
├── /data/
│   ├── /arb-bot/
│   │   ├── trades.json
│   │   ├── ml_settings.json       # Current ML parameters
│   │   ├── ml_history.json        # ML decision history
│   │   └── models/                # Runtime model files
│   │
│   ├── /spread-bot/
│   │   ├── trades.json
│   │   ├── ml_settings.json
│   │   └── ml_history.json
│   │
│   └── /portfolio/
│       ├── combined_metrics.json
│       └── insights_history.json
│
├── /.claude/
│   └── /skills/
│       ├── optimize-arb-bot.md
│       ├── optimize-spread-bot.md
│       ├── portfolio-insights.md
│       ├── add-new-bot.md
│       └── explain-ml-decision.md
│
├── Cargo.toml
├── ARCHITECTURE.md
├── MIGRATION_PLAN.md
└── SESSION_PROMPT.md
```

---

## ML Comparison Summary

| Bot | ML Approach | Why Best Fit |
|-----|-------------|--------------|
| **Arb Bot** | Bayesian Optimization + Multi-Armed Bandit | Fast threshold tuning, handles uncertainty, adapts per category |
| **Spread Bot** | LSTM + XGBoost + Q-Learning | Time series for prediction, classification for entry, RL for exit |
| **Portfolio** | Ensemble + LLM | Combines multiple signals, generates human explanations |

---

## Dashboard ML Panel (Per Bot)

```
┌─────────────────────────────────────────────────────────────────┐
│ ARB BOT - ML Insights                                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ Current Thresholds (Auto-Optimized):                           │
│ ┌─────────┬───────────┬───────────┬──────────┐                 │
│ │ Category│ Threshold │ Fill Rate │ Profit/Trade│              │
│ ├─────────┼───────────┼───────────┼──────────┤                 │
│ │ NBA     │ 0.45%     │ 92%       │ $1.23    │ ▲ Best         │
│ │ NFL     │ 0.70%     │ 71%       │ $0.87    │                 │
│ │ Crypto  │ 0.35%     │ 88%       │ $0.95    │                 │
│ │ EPL     │ 0.55%     │ 85%       │ $1.05    │                 │
│ └─────────┴───────────┴───────────┴──────────┘                 │
│                                                                 │
│ ML Recommendations:                                             │
│ ⚡ "Increase NBA position size - consistently outperforming"    │
│ ⚠️ "NFL fill rate dropped 15% - raised threshold automatically"│
│ 💡 "Crypto best between 2-6am UTC - consider time filter"       │
│                                                                 │
│ Model Confidence: 87% │ Last Retrained: 2 hours ago            │
└─────────────────────────────────────────────────────────────────┘
```

---

## How ML Training Works

```
┌─────────────────┐
│   Bot Running   │
│   Making Trades │
└────────┬────────┘
         │
         ▼ Logs every trade
┌─────────────────┐
│  trades.json    │
│  - entry price  │
│  - exit price   │
│  - fill status  │
│  - category     │
│  - timestamp    │
└────────┬────────┘
         │
         ▼ Every N trades or hourly
┌─────────────────┐
│  ML Retraining  │
│  - Load trades  │
│  - Extract features │
│  - Update model │
│  - Save new params │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ ml_settings.json│
│ (new thresholds)│
└────────┬────────┘
         │
         ▼ Bot loads new settings
┌─────────────────┐
│ Bot uses new    │
│ optimized params│
└─────────────────┘
```

---

## Next Steps

1. **Phase 1:** Complete multi-bot migration (MIGRATION_PLAN.md)
2. **Phase 2:** Add ML infrastructure to each bot
3. **Phase 3:** Build dashboard ML panels
4. **Phase 4:** Create Claude Code skills
5. **Phase 5:** Add LLM-powered insight generation
