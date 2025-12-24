//! Shared Market Client Library
//!
//! This library provides reusable API clients for Polymarket and Kalshi,
//! allowing multiple trading bots to share the same infrastructure code.

pub mod polymarket_clob;

// Re-export commonly used types
pub use polymarket_clob::{
    PolymarketAsyncClient,
    PreparedCreds,
    SharedAsyncClient,
    PolyFillAsync,
    OrderbookData,
    OrderbookLevel,
};
