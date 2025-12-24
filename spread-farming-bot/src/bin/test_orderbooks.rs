//! Test script to fetch real token IDs and test orderbook API
//!
//! This will:
//! 1. Fetch real market data from Polymarket Gamma API
//! 2. Get token IDs for Bitcoin and Ethereum 2025 markets
//! 3. Test orderbook API with each token
//! 4. Show which tokens have good liquidity

use anyhow::Result;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
struct GammaMarket {
    question: String,
    clob_token_ids: Vec<String>,
    outcomes: Vec<String>,
    #[serde(default)]
    volume: String,
}

#[derive(Debug, Deserialize)]
struct OrderbookData {
    bids: Vec<OrderLevel>,
    asks: Vec<OrderLevel>,
}

#[derive(Debug, Deserialize)]
struct OrderLevel {
    price: String,
    size: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Testing Polymarket Orderbook API with Real Token IDs ===\n");

    let client = reqwest::Client::new();

    // Test Bitcoin market
    println!("1. Fetching Bitcoin 2025 Price Market...");
    test_market(
        &client,
        "what-price-will-bitcoin-hit-in-2025",
        "Bitcoin 2025 Price",
    ).await?;

    println!("\n{}\n", "=".repeat(80));

    // Test Ethereum market
    println!("2. Fetching Ethereum 2025 Price Market...");
    test_market(
        &client,
        "what-price-will-ethereum-hit-in-2025",
        "Ethereum 2025 Price",
    ).await?;

    println!("\n=== Testing Complete ===");
    println!("\n💡 Next Steps:");
    println!("   1. Pick 2-3 tokens with good liquidity (tight spread, good size)");
    println!("   2. Update src/config.rs with those token IDs");
    println!("   3. Run the bot in DRY_RUN mode to test");

    Ok(())
}

async fn test_market(client: &reqwest::Client, slug: &str, name: &str) -> Result<()> {
    // Fetch market from Gamma API
    let url = format!("https://gamma-api.polymarket.com/markets?slug={}", slug);
    let resp = client.get(&url).send().await?;

    if !resp.status().is_success() {
        println!("   ✗ Failed to fetch market: {}", resp.status());
        return Ok(());
    }

    let markets: Vec<GammaMarket> = resp.json().await?;

    if markets.is_empty() {
        println!("   ✗ No market found for slug: {}", slug);
        return Ok(());
    }

    let market = &markets[0];
    println!("   ✓ Found market: {}", market.question);
    println!("   Volume: ${}", market.volume);
    println!("   Outcomes: {}", market.outcomes.len());
    println!();

    // Test orderbook for each token
    for (i, token_id) in market.clob_token_ids.iter().enumerate() {
        let outcome = market.outcomes.get(i).map(|s| s.as_str()).unwrap_or("Unknown");

        println!("   Testing outcome: {}", outcome);
        println!("   Token ID: {}", token_id);

        // Fetch orderbook
        match fetch_orderbook(client, token_id).await {
            Ok(book) => {
                let best_bid = book.bids.first()
                    .and_then(|l| l.price.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let best_ask = book.asks.first()
                    .and_then(|l| l.price.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let bid_size = book.bids.first()
                    .and_then(|l| l.size.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let ask_size = book.asks.first()
                    .and_then(|l| l.size.parse::<f64>().ok())
                    .unwrap_or(0.0);

                let spread_bps = if best_bid > 0.0 && best_ask > 0.0 {
                    ((best_ask - best_bid) / best_bid * 10000.0) as i32
                } else {
                    0
                };

                println!("   ✓ Orderbook:");
                println!("      Best Bid: {:.4} (size: {:.2})", best_bid, bid_size);
                println!("      Best Ask: {:.4} (size: {:.2})", best_ask, ask_size);
                println!("      Spread: {} bps ({:.2}%)", spread_bps, spread_bps as f64 / 100.0);

                // Liquidity assessment
                if best_bid == 0.0 || best_ask == 0.0 {
                    println!("      ⚠️  WARNING: No liquidity!");
                } else if spread_bps > 500 {
                    println!("      ⚠️  WARNING: Wide spread (>5%) - low liquidity");
                } else if bid_size < 10.0 || ask_size < 10.0 {
                    println!("      ⚠️  WARNING: Thin orderbook (<$10)");
                } else {
                    println!("      ✅ GOOD LIQUIDITY - suitable for market making");
                }
            }
            Err(e) => {
                println!("   ✗ Failed to fetch orderbook: {}", e);
            }
        }

        println!();
    }

    Ok(())
}

async fn fetch_orderbook(client: &reqwest::Client, token_id: &str) -> Result<OrderbookData> {
    let url = format!("https://clob.polymarket.com/book?token_id={}", token_id);
    let resp = client.get(&url).send().await?;

    if !resp.status().is_success() {
        anyhow::bail!("HTTP {}", resp.status());
    }

    Ok(resp.json().await?)
}
