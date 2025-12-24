# How to Find Real Polymarket Market Slugs

## Step 1: Visit Polymarket Crypto Markets

Go to: **https://polymarket.com/predictions/crypto-prices**

## Step 2: Find High-Volume Markets

Look for markets with:
- ✅ **High volume** ($5M+ daily volume)
- ✅ **High liquidity** ($3M+ liquidity)
- ✅ **Long time horizon** (resolves in 2025 or later)
- ✅ **Clear binary outcome** (Yes/No question)

## Step 3: Extract Market Slugs

Based on the search results, here are the **highest volume markets** to target:

### Bitcoin Markets (Highest Priority)

1. **"What price will Bitcoin hit in 2025?"**
   - Volume: $147M total, $5M daily
   - Liquidity: $7M
   - Slug: Visit https://polymarket.com and search for this market
   - Copy the slug from the URL (e.g., `/markets/bitcoin-price-2025`)

2. **"Will Satoshi move any Bitcoin in 2025?"**
   - Volume: $21M
   - Slug: Search on Polymarket and copy URL

### Ethereum Markets

3. **"What price will Ethereum hit in 2025?"**
   - Volume: $64M total, $1M daily
   - Liquidity: $3M
   - Slug: Search on Polymarket and copy URL

4. **"Will Ethereum reach $7,000 in 2025?"**
   - Active market with good liquidity
   - Slug: Search on Polymarket

## Step 4: Get Token IDs (Advanced)

Once you have the market slug, you need the YES/NO token IDs.

### Method 1: Browser Developer Tools
1. Open the market page on Polymarket
2. Open browser DevTools (F12)
3. Go to Network tab
4. Look for API calls to `clob.polymarket.com`
5. Find `token_id` in the response

### Method 2: Use Gamma API
```bash
curl "https://gamma-api.polymarket.com/markets?slug=YOUR_MARKET_SLUG"
```

Look for `clobTokenIds` in the response.

## Step 5: Update config.rs

Edit `/home/user/trading-bots/spread-farming-bot/src/config.rs`:

```rust
pub const TARGET_MARKETS: &[(&str, &str)] = &[
    // Replace these with real slugs from Polymarket
    ("bitcoin-price-2025", "Bitcoin Price 2025"),
    ("ethereum-price-2025", "Ethereum Price 2025"),
    ("satoshi-move-bitcoin-2025", "Satoshi Move Bitcoin 2025"),
    // ... add more markets
];
```

## Example: Finding a Market Slug

1. Go to https://polymarket.com
2. Search for "Bitcoin 2025"
3. Click on "What price will Bitcoin hit in 2025?"
4. URL becomes: `https://polymarket.com/event/bitcoin-2025-price-prediction`
5. Slug is: `bitcoin-2025-price-prediction`

## Quick Reference: Common Slug Patterns

Polymarket slugs usually follow these patterns:
- `bitcoin-price-2025`
- `ethereum-reach-7000-2025`
- `btc-above-100k-december-2024`
- `eth-price-prediction-2025`

## What to Look For

**Good markets for spread farming:**
- ✅ > $1M daily volume
- ✅ > $1M liquidity
- ✅ Tight spread (< 5%)
- ✅ Stable mid-price
- ✅ Long time horizon

**Avoid:**
- ❌ Low volume (< $100K daily)
- ❌ Wide spread (> 10%)
- ❌ Resolves soon (< 1 month)
- ❌ Highly volatile

## Need Help?

After finding the markets, update `config.rs` and rebuild:
```bash
cd /home/user/trading-bots/spread-farming-bot
nano src/config.rs  # Update TARGET_MARKETS
cargo build --release
```
