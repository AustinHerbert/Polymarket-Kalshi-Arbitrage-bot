//! Type definitions for spread farming bot

use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Market information
#[derive(Debug, Clone)]
pub struct Market {
    /// Market slug (unique identifier)
    pub slug: Arc<str>,
    
    /// Human-readable description
    pub description: Arc<str>,
    
    /// YES token ID
    pub yes_token_id: Arc<str>,
    
    /// NO token ID
    pub no_token_id: Arc<str>,
    
    /// Current orderbook state
    pub orderbook: OrderbookState,
}

/// Orderbook state for a single side (YES or NO)
#[derive(Debug, Clone, Copy, Default)]
pub struct OrderbookState {
    /// Best bid price (0.0-1.0)
    pub best_bid: f64,
    
    /// Best ask price (0.0-1.0)
    pub best_ask: f64,
    
    /// Mid price (average of bid/ask)
    pub mid_price: f64,
    
    /// Last update timestamp
    pub last_update: u64,
}

impl OrderbookState {
    pub fn new(best_bid: f64, best_ask: f64) -> Self {
        Self {
            best_bid,
            best_ask,
            mid_price: (best_bid + best_ask) / 2.0,
            last_update: chrono::Utc::now().timestamp() as u64,
        }
    }
    
    pub fn is_valid(&self) -> bool {
        self.best_bid > 0.0 && self.best_ask > 0.0 && self.best_bid < self.best_ask
    }
}

/// Active limit order
#[derive(Debug, Clone)]
pub struct ActiveOrder {
    /// Order ID from Polymarket
    pub order_id: String,
    
    /// Token ID (YES or NO)
    pub token_id: Arc<str>,
    
    /// Side (BUY or SELL)
    pub side: OrderSide,
    
    /// Price (0.0-1.0)
    pub price: f64,
    
    /// Size in contracts
    pub size: f64,
    
    /// Timestamp when placed
    pub placed_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl OrderSide {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        }
    }
}

/// Inventory position for a market
#[derive(Debug, Clone, Default)]
pub struct Position {
    /// YES contracts held
    pub yes_contracts: f64,
    
    /// NO contracts held
    pub no_contracts: f64,
    
    /// Total cost basis in USD
    pub cost_basis: f64,
    
    /// Realized P&L in USD
    pub realized_pnl: f64,
}

impl Position {
    /// Net exposure (positive = long YES, negative = long NO)
    pub fn net_exposure(&self) -> f64 {
        self.yes_contracts - self.no_contracts
    }
    
    /// Total value at current mid-price
    pub fn market_value(&self, mid_price: f64) -> f64 {
        self.yes_contracts * mid_price + self.no_contracts * (1.0 - mid_price)
    }
    
    /// Unrealized P&L
    pub fn unrealized_pnl(&self, mid_price: f64) -> f64 {
        self.market_value(mid_price) - self.cost_basis
    }
}
