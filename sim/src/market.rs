//! Market system

use crate::grid::PixelCoord;
use crate::building::ResourceType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Sell offer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellOffer {
    pub id: Uuid,
    pub seller_id: u32,
    pub resource: ResourceType,
    pub quantity: u32,
    pub price_per_unit: u32, // in Gold
    pub total_gold: u32,
    pub duration_hours: u8, // 1-168
    pub created_tick: u64,
    pub market_building_level: u8, // 0=direct, 1=local, 2=global, 3=premium
    pub remaining_quantity: u32,
}

/// Market system
pub struct MarketSystem;

impl MarketSystem {
    /// Calculate distance-based fee
    pub fn fee_percent(distance: u32, seller_market_level: u8, buyer_market_level: u8) -> u8 {
        let max_level = seller_market_level.max(buyer_market_level);
        if max_level >= 3 {
            return 0; // L3 = zero fees
        }

        let base_fee = match distance {
            0..=500 => 0,
            501..=2000 => 2,
            2001..=5000 => 5,
            _ => 8,
        };

        // Workers reduce fee (0.5% per worker, min 0%)
        let worker_reduction = seller_market_level.min(base_fee);
        base_fee.saturating_sub(worker_reduction)
    }

    /// Calculate Manhattan distance between two pixels
    pub fn distance(a: PixelCoord, b: PixelCoord) -> u32 {
        a.x.abs_diff(b.x) + a.y.abs_diff(b.y)
    }

    /// Execute trade (atomic swap)
    pub fn execute_trade(
        offer: &mut SellOffer,
        buyer_id: u32,
        quantity: u32,
    ) -> Option<TradeResult> {
        if quantity > offer.remaining_quantity {
            return None;
        }

        let gold = quantity * offer.price_per_unit;
        let fee = (gold * Self::fee_percent(0, offer.market_building_level, 0) as u32) / 100;
        let seller_gold = gold - fee;

        offer.remaining_quantity -= quantity;

        Some(TradeResult {
            buyer_id,
            seller_id: offer.seller_id,
            resource: offer.resource,
            quantity,
            gold_paid: gold,
            fee,
            seller_received: seller_gold,
        })
    }
}

/// Trade execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeResult {
    pub buyer_id: u32,
    pub seller_id: u32,
    pub resource: ResourceType,
    pub quantity: u32,
    pub gold_paid: u32,
    pub fee: u32,
    pub seller_received: u32,
}

/// Market state per player
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerMarket {
    pub offers: HashMap<Uuid, SellOffer>,
    pub trade_history: Vec<TradeResult>,
}