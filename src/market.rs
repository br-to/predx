use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;

/// 検索結果1件分のデータ
#[derive(Debug, Clone, Serialize)]
pub struct MarketItem {
    pub id: String,
    pub platform: &'static str,
    pub title: String,
    pub probability: f64,
    pub volume: f64,
    pub active: bool,
}

/// Polymarket/Kalshi両方が実装する統一インターフェース
#[async_trait]
pub trait Market {
    fn name(&self) -> &str;
    async fn search(&self, query: &str) -> Result<Vec<MarketItem>>;
    async fn get_by_id(&self, id: &str) -> Result<MarketItem>;
}
