use anyhow::Result;
use async_trait::async_trait;

use crate::market::{Market, MarketItem};

pub struct Polymarket {
    client: reqwest::Client,
}

impl Polymarket {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Market for Polymarket {
    fn name(&self) -> &str {
        "Polymarket"
    }

    async fn search(&self, _query: &str) -> Result<Vec<MarketItem>> {
        // PR2で実装
        Ok(vec![])
    }
}
