//! Standalone monitoring dashboard binary
//!
//! Run with: cargo run --bin monitor
//! Or: cargo build --release && ./target/release/monitor

#[path = "../monitor_dashboard.rs"]
mod monitor_dashboard;

#[tokio::main]
async fn main() {
    println!("Starting Arbitrage Bot Monitor...");
    monitor_dashboard::run_monitor_server().await;
}
