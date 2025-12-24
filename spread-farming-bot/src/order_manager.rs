//! Order management for limit orders

use anyhow::{Result, anyhow};
use dashmap::DashMap;
use std::sync::Arc;
use tracing::{info, warn};

use crate::config::SpreadFarmingConfig;
use crate::types::{ActiveOrder, OrderSide, Position};
use shared_market_client::SharedAsyncClient;

/// Manages limit orders for all markets
pub struct OrderManager {
    /// Polymarket client
    poly_client: Arc<SharedAsyncClient>,
    
    /// Configuration
    config: SpreadFarmingConfig,
    
    /// Active orders by token_id
    active_orders: DashMap<String, Vec<ActiveOrder>>,
    
    /// Positions by token_id
    positions: DashMap<String, Position>,
}

impl OrderManager {
    pub fn new(poly_client: Arc<SharedAsyncClient>, config: SpreadFarmingConfig) -> Self {
        Self {
            poly_client,
            config,
            active_orders: DashMap::new(),
            positions: DashMap::new(),
        }
    }
    
    /// Place a limit order
    pub async fn place_order(
        &self,
        token_id: &str,
        side: OrderSide,
        price: f64,
        size: f64,
    ) -> Result<String> {
        if self.config.dry_run {
            let fake_id = format!("DRY_{}_{}_{}", token_id, side.as_str(), price);
            info!("[ORDER] 🏃 DRY RUN - Would place {} {} @ {:.4} x {:.2} contracts",
                  side.as_str(), token_id, price, size);
            return Ok(fake_id);
        }
        
        // Place order via Polymarket API
        let order_id = match side {
            OrderSide::Buy => {
                let result = self.poly_client.buy_fak(token_id, price, size).await?;
                result.order_id
            }
            OrderSide::Sell => {
                let result = self.poly_client.sell_fak(token_id, price, size).await?;
                result.order_id
            }
        };
        
        info!("[ORDER] ✅ Placed {} {} @ {:.4} x {:.2} → Order ID: {}",
              side.as_str(), token_id, price, size, order_id);
        
        // Track order
        let order = ActiveOrder {
            order_id: order_id.clone(),
            token_id: token_id.into(),
            side,
            price,
            size,
            placed_at: chrono::Utc::now().timestamp() as u64,
        };
        
        self.active_orders
            .entry(token_id.to_string())
            .or_insert_with(Vec::new)
            .push(order);
        
        Ok(order_id)
    }
    
    /// Calculate optimal order size given current position
    pub fn calculate_order_size(
        &self,
        token_id: &str,
        mid_price: f64,
        side: OrderSide,
    ) -> f64 {
        let position = self.positions
            .get(token_id)
            .map(|p| p.clone())
            .unwrap_or_default();
        
        // Base size in USD
        let base_size_usd = self.config.min_order_size_usd;
        
        // Calculate how many contracts that is
        let price = match side {
            OrderSide::Buy => mid_price * 0.98, // Buy below mid
            OrderSide::Sell => mid_price * 1.02, // Sell above mid
        };
        
        let contracts = base_size_usd / price;
        
        // Check position limits
        let max_contracts = self.config.max_position_per_market_usd / price;
        
        match side {
            OrderSide::Buy => {
                // Buying increases position
                let room = max_contracts - position.yes_contracts;
                contracts.min(room).max(0.0)
            }
            OrderSide::Sell => {
                // Need inventory to sell
                position.yes_contracts.min(contracts)
            }
        }
    }
    
    /// Update position after fill
    pub fn update_position(
        &self,
        token_id: &str,
        side: OrderSide,
        filled_contracts: f64,
        fill_price: f64,
    ) {
        let mut position = self.positions
            .entry(token_id.to_string())
            .or_insert_with(Position::default);
        
        match side {
            OrderSide::Buy => {
                position.yes_contracts += filled_contracts;
                position.cost_basis += filled_contracts * fill_price;
            }
            OrderSide::Sell => {
                let proceeds = filled_contracts * fill_price;
                let cost = (filled_contracts / position.yes_contracts.max(1.0)) * position.cost_basis;
                position.yes_contracts -= filled_contracts;
                position.cost_basis -= cost;
                position.realized_pnl += proceeds - cost;
            }
        }
    }
    
    /// Get current position for a token
    pub fn get_position(&self, token_id: &str) -> Position {
        self.positions
            .get(token_id)
            .map(|p| p.clone())
            .unwrap_or_default()
    }
    
    /// Get summary of all positions
    pub fn get_positions_summary(&self) -> Vec<(String, Position)> {
        self.positions
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }
}
