use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;

use crate::market::{Market, MarketItem};

const SEARCH_URL: &str =
    "https://api.elections.kalshi.com/v1/search/series";
const PAGE_SIZE: u32 = 24;

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

    async fn search(&self, query: &str) -> Result<Vec<MarketItem>> {
        let resp: SearchResponse = self
            .client
            .get(SEARCH_URL)
            .query(&[
                ("query", query),
                ("order_by", "querymatch"),
                ("reverse", "false"),
                ("page_size", &PAGE_SIZE.to_string()),
            ])
            .send()
            .await?
            .json()
            .await?;

        let mut items = Vec::new();
        for contract in resp.current_page {
            for market in contract.markets {
                let probability = market.last_price as f64 / 100.0;
                let volume = market.volume as f64;
                let title = if market.yes_subtitle.is_empty() {
                    contract.event_title.clone()
                } else {
                    format!("{} — {}", contract.event_title, market.yes_subtitle)
                };
                items.push(MarketItem {
                    id: market.ticker,
                    platform: "kalshi",
                    title,
                    probability,
                    volume,
                    active: market.result.is_empty(),
                });
            }
        }
        Ok(items)
    }
}

#[derive(Deserialize)]
struct SearchResponse {
    current_page: Vec<Contract>,
}

#[derive(Deserialize)]
struct Contract {
    event_title: String,
    #[serde(default)]
    markets: Vec<KalshiMarket>,
}

#[derive(Deserialize)]
struct KalshiMarket {
    #[serde(default)]
    ticker: String,
    #[serde(default)]
    last_price: u32,
    #[serde(default)]
    volume: u64,
    #[serde(default)]
    yes_subtitle: String,
    #[serde(default)]
    result: String,
}
