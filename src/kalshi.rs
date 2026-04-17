use anyhow::Result;
use async_trait::async_trait;

use crate::market::{Market, MarketItem};

pub struct Kalshi {
    client: reqwest::Client,
}

impl Kalshi {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Market for Kalshi {
    fn name(&self) -> &str {
        "Kalshi"
    }

    async fn search(&self, _query: &str) -> Result<Vec<MarketItem>> {
        // PR3で実装
        Ok(vec![])
    }
}
