//! Prediction Market Arbitrage Trading System
//!
//! A high-performance, production-ready arbitrage trading system for cross-platform
//! prediction markets with real-time price monitoring and execution.

pub mod cache;
pub mod circuit_breaker;
pub mod config;
pub mod discovery;
pub mod execution;
pub mod kalshi;
pub mod metrics;
pub mod polymarket;
pub mod polymarket_clob;
pub mod position_tracker;
pub mod priority_config;
pub mod priority_queue;
pub mod types;