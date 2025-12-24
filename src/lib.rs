//! Prediction Market Arbitrage Trading System
//!
//! A high-performance, production-ready arbitrage trading system for cross-platform
//! prediction markets with real-time price monitoring and execution.

pub mod balance_tracker;
pub mod cache;
pub mod circuit_breaker;
pub mod config;
pub mod crypto_discovery;
pub mod discovery;
pub mod execution;
pub mod insights;
pub mod kalshi;
pub mod metrics;
pub mod ml_optimizer;
pub mod opportunity_log;
pub mod polymarket;
pub mod polymarket_clob;
pub mod position_tracker;
pub mod priority_config;
pub mod priority_queue;
pub mod trade_log;
pub mod types;