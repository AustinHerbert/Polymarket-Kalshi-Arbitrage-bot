//! Core market making engine

use anyhow::{Result, anyhow};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{info, warn, error};

use crate::config::{SpreadFarmingConfig, TARGET_MARKETS};
use crate::order_manager::OrderManager;
use crate::types::{Market, OrderbookState, OrderSide};
use shared_market_client::SharedAsyncClient;

/// Core market making engine
pub struct MarketMaker {
    /// Polymarket client
    poly_client: Arc<SharedAsyncClient>,
    
    /// Configuration
    config: SpreadFarmingConfig,
    
    /// Order manager
    order_manager: Arc<OrderManager>,
    
    /// Tracked markets
    markets: Vec<Market>,
}

impl MarketMaker {
    pub fn new(poly_client: Arc<SharedAsyncClient>, config: SpreadFarmingConfig) -> Self {
        let order_manager = Arc::new(OrderManager::new(poly_client.clone(), config.clone()));
        
        Self {
            poly_client,
            config,
            order_manager,
            markets: Vec::new(),
        }
    }
    
    /// Initialize markets (fetch token IDs from Polymarket)
    pub async fn initialize(&self) -> Result<()> {
        info!("[INIT] Fetching market data from Polymarket...");
        
        // In production, you'd fetch real token IDs from Polymarket API
        // For now, using placeholder logic
        for (slug, description) in TARGET_MARKETS {
            info!("   → Initializing market: {}", description);
            
            // TODO: Fetch real token IDs from Polymarket API
            // For now, using mock data
            let yes_token_id = format!("yes_token_{}", slug);
            let no_token_id = format!("no_token_{}", slug);
            
            info!("     YES token: {}", yes_token_id);
            info!("     NO token: {}", no_token_id);
        }
        
        Ok(())
    }
    
    /// Main market making loop
    pub async fn run(&self) -> Result<()> {
        let mut update_timer = interval(Duration::from_millis(self.config.update_interval_ms));
        let mut stats_timer = interval(Duration::from_secs(60));
        
        info!("[MM] Market making engine started");
        info!("[MM] Update interval: {}ms", self.config.update_interval_ms);
        info!("[MM] Spread: {} bps ({:.2}%)", self.config.spread_bps, self.config.spread_bps as f64 / 100.0);
        
        loop {
            tokio::select! {
                _ = update_timer.tick() => {
                    if let Err(e) = self.update_orders().await {
                        error!("[MM] Error updating orders: {}", e);
                    }
                }
                
                _ = stats_timer.tick() => {
                    self.print_statistics();
                }
            }
        }
    }
    
    /// Update all orders (main market making logic)
    async fn update_orders(&self) -> Result<()> {
        for (slug, description) in TARGET_MARKETS {
            // TODO: Fetch real token IDs from Polymarket API using the market slug
            // For now using placeholder token IDs (replace with real ones!)
            let token_id = format!("yes_token_{}", slug);

            // Fetch current orderbook state
            let orderbook = self.fetch_orderbook(&token_id).await?;
            
            if !orderbook.is_valid() {
                warn!("[MM] Invalid orderbook for {}: {:?}", description, orderbook);
                continue;
            }
            
            // Calculate quote prices
            let spread_fraction = (self.config.spread_bps as f64) / 10000.0;
            let bid_price = orderbook.mid_price * (1.0 - spread_fraction / 2.0);
            let ask_price = orderbook.mid_price * (1.0 + spread_fraction / 2.0);
            
            // Calculate order sizes
            let bid_size = self.order_manager.calculate_order_size(
                &token_id,
                orderbook.mid_price,
                OrderSide::Buy,
            );
            
            let ask_size = self.order_manager.calculate_order_size(
                &token_id,
                orderbook.mid_price,
                OrderSide::Sell,
            );
            
            // Place orders if size > 0
            if bid_size > 1.0 {
                if let Err(e) = self.order_manager.place_order(
                    &token_id,
                    OrderSide::Buy,
                    bid_price,
                    bid_size,
                ).await {
                    error!("[MM] Failed to place BUY order for {}: {}", description, e);
                }
            }
            
            if ask_size > 1.0 {
                if let Err(e) = self.order_manager.place_order(
                    &token_id,
                    OrderSide::Sell,
                    ask_price,
                    ask_size,
                ).await {
                    error!("[MM] Failed to place SELL order for {}: {}", description, e);
                }
            }
            
            info!("[MM] {} | Mid: {:.4} | Quotes: {:.4}x{:.2} @ {:.4}x{:.2}",
                  description,
                  orderbook.mid_price,
                  bid_price, bid_size,
                  ask_price, ask_size);
        }
        
        Ok(())
    }
    
    /// Fetch current orderbook state from Polymarket API
    async fn fetch_orderbook(&self, token_id: &str) -> Result<OrderbookState> {
        // Fetch real orderbook data from Polymarket
        let book = self.poly_client.get_orderbook(token_id).await?;

        // Parse best bid and ask
        let best_bid = book.bids
            .first()
            .and_then(|level| level.price.parse::<f64>().ok())
            .unwrap_or(0.0);

        let best_ask = book.asks
            .first()
            .and_then(|level| level.price.parse::<f64>().ok())
            .unwrap_or(0.0);

        Ok(OrderbookState::new(best_bid, best_ask))
    }
    
    /// Print statistics
    fn print_statistics(&self) {
        info!("📊 Statistics:");
        
        let positions = self.order_manager.get_positions_summary();
        
        if positions.is_empty() {
            info!("   No open positions");
        } else {
            let mut total_pnl = 0.0;
            
            for (token_id, position) in positions {
                total_pnl += position.realized_pnl;
                
                if position.yes_contracts > 0.1 || position.no_contracts > 0.1 {
                    info!("   {} | YES: {:.2} NO: {:.2} | P&L: ${:.2}",
                          &token_id[..20],
                          position.yes_contracts,
                          position.no_contracts,
                          position.realized_pnl);
                }
            }
            
            info!("   Total realized P&L: ${:.2}", total_pnl);
        }
    }
}
