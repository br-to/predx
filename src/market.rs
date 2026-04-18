use anyhow::Result;
use async_trait::async_trait;

/// 検索結果1件分のデータ
#[derive(Debug, Clone)]
pub struct MarketItem {
    pub title: String,
    pub probability: f64,
    pub volume: f64,
}

/// Polymarket/Kalshi両方が実装する統一インターフェース
#[async_trait]
pub trait Market {
    fn name(&self) -> &str;
    async fn search(&self, query: &str) -> Result<Vec<MarketItem>>;
}
