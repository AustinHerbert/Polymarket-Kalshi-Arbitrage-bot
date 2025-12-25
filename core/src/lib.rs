//! Trading Core - Shared infrastructure for Polymarket/Kalshi trading bots
//!
//! This crate provides common functionality used by all trading bots:
//! - API clients (Kalshi REST/WS, Polymarket WS/CLOB)
//! - Configuration management
//! - Common types and utilities

pub mod api;
pub mod config;
pub mod utils;

// Re-export commonly used items at crate root
pub use api::kalshi;
pub use api::polymarket;
pub use api::polymarket_clob;
pub use utils::types;
pub use utils::cache;
pub use utils::NanoClock;
