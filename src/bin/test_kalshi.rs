//! Test Kalshi API authentication

use anyhow::Result;
use prediction_market_arbitrage::kalshi::KalshiConfig;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    println!("=== Kalshi API Authentication Test ===\n");

    // Load config from environment
    let config = KalshiConfig::from_env()?;
    println!("API key: {}...", &config.api_key_id[..8]);

    // Test 1: Sign a message and show details
    let test_message = "1735000000000GET/trade-api/ws/v2";
    println!("\nTest message: '{}'", test_message);

    let signature = config.sign(test_message)?;
    println!("Signature length: {}", signature.len());
    println!("Signature (first 50): {}", &signature[..50.min(signature.len())]);

    // Test 2: Try REST API with real timestamp
    println!("\n=== Testing REST API ===");

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let path = "/trade-api/v2/portfolio/balance";
    let message = format!("{}GET{}", timestamp, path);
    println!("Message: '{}'", message);

    let signature = config.sign(&message)?;
    println!("Signature (first 50): {}", &signature[..50]);

    let http = reqwest::Client::new();
    let resp = http
        .get(format!("https://api.elections.kalshi.com{}", path))
        .header("KALSHI-ACCESS-KEY", &config.api_key_id)
        .header("KALSHI-ACCESS-SIGNATURE", &signature)
        .header("KALSHI-ACCESS-TIMESTAMP", timestamp.to_string())
        .send()
        .await?;

    let status = resp.status();
    let body = resp.text().await?;

    if status.is_success() {
        println!("\n✅ REST API SUCCESS!");
        println!("Response: {}", body);
    } else {
        println!("\n❌ REST API FAILED!");
        println!("Status: {}", status);
        println!("Response: {}", body);
    }

    // Test 3: Try WebSocket connection
    println!("\n=== Testing WebSocket ===");

    use tokio_tungstenite::{connect_async, tungstenite::http::Request};

    let ws_timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string();

    let ws_message = format!("{}GET/trade-api/ws/v2", ws_timestamp);
    println!("WS Message: '{}'", ws_message);

    let ws_signature = config.sign(&ws_message)?;
    println!("WS Signature (first 50): {}", &ws_signature[..50]);

    let request = Request::builder()
        .uri("wss://api.elections.kalshi.com/trade-api/ws/v2")
        .header("Host", "api.elections.kalshi.com")
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Key", tokio_tungstenite::tungstenite::handshake::client::generate_key())
        .header("KALSHI-ACCESS-KEY", &config.api_key_id)
        .header("KALSHI-ACCESS-SIGNATURE", &ws_signature)
        .header("KALSHI-ACCESS-TIMESTAMP", &ws_timestamp)
        .body(())?;

    match connect_async(request).await {
        Ok((ws_stream, _)) => {
            println!("\n✅ WebSocket SUCCESS!");
            drop(ws_stream);
        }
        Err(e) => {
            println!("\n❌ WebSocket FAILED!");
            println!("Error: {:?}", e);
        }
    }

    Ok(())
}
