//! Core market making engine

use anyhow::Result;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{info, warn, error};

use crate::config::{SpreadFarmingConfig, TARGET_MARKETS, MarketConfig};
use crate::order_manager::OrderManager;
use crate::types::{OrderbookState, OrderSide};
use shared_market_client::SharedAsyncClient;

/// Core market making engine
pub struct MarketMaker {
    /// Polymarket client
    poly_client: Arc<SharedAsyncClient>,

    /// Configuration
    config: SpreadFarmingConfig,

    /// Order manager
    order_manager: Arc<OrderManager>,
}

impl MarketMaker {
    pub fn new(poly_client: Arc<SharedAsyncClient>, config: SpreadFarmingConfig) -> Self {
        let order_manager = Arc::new(OrderManager::new(poly_client.clone(), config.clone()));

        Self {
            poly_client,
            config,
            order_manager,
        }
    }

    /// Initialize markets - verify token IDs are valid
    pub async fn initialize(&self) -> Result<()> {
        info!("[INIT] Verifying market data from Polymarket...");

        for market in TARGET_MARKETS {
            info!("   → Initializing market: {}", market.description);
            info!("     YES token: {}...", &market.yes_token[..40]);
            info!("     NO token: {}...", &market.no_token[..40]);

            // Verify the orderbook exists
            match self.fetch_orderbook(market.yes_token).await {
                Ok(book) => {
                    if book.is_valid() {
                        info!("     ✓ Orderbook valid: mid={:.4}", book.mid_price);
                    } else {
                        warn!("     ⚠ Orderbook empty or invalid");
                    }
                }
                Err(e) => {
                    error!("     ✗ Failed to fetch orderbook: {}", e);
                }
            }
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
        for market in TARGET_MARKETS {
            // Use the REAL token IDs from config
            let yes_token = market.yes_token;
            let no_token = market.no_token;

            // Fetch current orderbook state for YES token
            let orderbook = match self.fetch_orderbook(yes_token).await {
                Ok(book) => book,
                Err(e) => {
                    warn!("[MM] {} - Failed to fetch orderbook: {}", market.name, e);
                    continue;
                }
            };

            if !orderbook.is_valid() {
                warn!("[MM] {} - Invalid orderbook: bid={:.4} ask={:.4}",
                      market.name, orderbook.best_bid, orderbook.best_ask);
                continue;
            }

            // Calculate quote prices based on spread
            let spread_fraction = (self.config.spread_bps as f64) / 10000.0;
            let half_spread = spread_fraction / 2.0;

            // Place bid below mid, ask above mid
            let bid_price = orderbook.mid_price * (1.0 - half_spread);
            let ask_price = orderbook.mid_price * (1.0 + half_spread);

            // Ensure prices are within valid range [0.01, 0.99]
            let bid_price = bid_price.max(0.01).min(0.99);
            let ask_price = ask_price.max(0.01).min(0.99);

            // Calculate order sizes
            let bid_size = self.order_manager.calculate_order_size(
                yes_token,
                orderbook.mid_price,
                OrderSide::Buy,
            );

            let ask_size = self.order_manager.calculate_order_size(
                yes_token,
                orderbook.mid_price,
                OrderSide::Sell,
            );

            // Log the quotes
            info!("[MM] {} | Mid: {:.4} | Bid: {:.4}x${:.2} | Ask: {:.4}x${:.2}",
                  market.name,
                  orderbook.mid_price,
                  bid_price, bid_size,
                  ask_price, ask_size);

            // Place orders if size > minimum
            if bid_size >= self.config.min_order_size_usd {
                if let Err(e) = self.order_manager.place_order(
                    yes_token,
                    OrderSide::Buy,
                    bid_price,
                    bid_size,
                ).await {
                    error!("[MM] {} - Failed to place BUY: {}", market.name, e);
                }
            }

            if ask_size >= self.config.min_order_size_usd {
                if let Err(e) = self.order_manager.place_order(
                    yes_token,
                    OrderSide::Sell,
                    ask_price,
                    ask_size,
                ).await {
                    error!("[MM] {} - Failed to place SELL: {}", market.name, e);
                }
            }
        }

        Ok(())
    }

    /// Fetch current orderbook state from Polymarket API
    async fn fetch_orderbook(&self, token_id: &str) -> Result<OrderbookState> {
        let book = self.poly_client.get_orderbook(token_id).await?;

        // Parse best bid (highest buy order)
        let best_bid = book.bids
            .first()
            .and_then(|level| level.price.parse::<f64>().ok())
            .unwrap_or(0.0);

        // Parse best ask (lowest sell order)
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
                    info!("   {}... | YES: {:.2} NO: {:.2} | P&L: ${:.2}",
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
