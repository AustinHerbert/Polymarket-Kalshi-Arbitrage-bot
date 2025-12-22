//! Simple test to verify Kalshi API credentials work

use anyhow::Result;
use prediction_market_arbitrage::kalshi::KalshiApiClient;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    println!("Testing Kalshi API credentials...\n");

    // Load config from environment
    let config = prediction_market_arbitrage::kalshi::KalshiConfig::from_env()?;
    println!("✅ API key loaded: {}...", &config.api_key_id[..8]);

    // Create client
    let client = KalshiApiClient::new(config);

    // Try to fetch exchange status (public endpoint, but we'll sign it anyway)
    println!("\nTesting REST API connection...");

    let url = "https://api.elections.kalshi.com/trade-api/v2/exchange/status";
    let resp = reqwest::get(url).await?;
    println!("Exchange status: {}", resp.text().await?);

    // Try an authenticated endpoint
    println!("\nTesting authenticated endpoint (portfolio/balance)...");

    let timestamp_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let path = "/portfolio/balance";
    let full_path = format!("/trade-api/v2{}", path);
    let signature = client.config.sign(&format!("{}GET{}", timestamp_ms, full_path))?;

    let http = reqwest::Client::new();
    let resp = http
        .get(format!("https://api.elections.kalshi.com/trade-api/v2{}", path))
        .header("KALSHI-ACCESS-KEY", &client.config.api_key_id)
        .header("KALSHI-ACCESS-SIGNATURE", &signature)
        .header("KALSHI-ACCESS-TIMESTAMP", timestamp_ms.to_string())
        .send()
        .await?;

    let status = resp.status();
    let body = resp.text().await?;

    if status.is_success() {
        println!("✅ Authentication successful!");
        println!("Balance response: {}", body);
    } else {
        println!("❌ Authentication failed!");
        println!("Status: {}", status);
        println!("Response: {}", body);
    }

    Ok(())
}
