use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;

use crate::market::{Market, MarketItem};

const EVENTS_URL: &str = "https://api.elections.kalshi.com/trade-api/v2/events";
const PAGE_LIMIT: u32 = 200;
const MAX_PAGES: u32 = 5;

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
        let query_lower = query.to_lowercase();
        let mut items = Vec::new();
        let mut cursor: Option<String> = None;

        for _ in 0..MAX_PAGES {
            let mut req = self
                .client
                .get(EVENTS_URL)
                .query(&[
                    ("status", "open"),
                    ("with_nested_markets", "true"),
                    ("limit", &PAGE_LIMIT.to_string()),
                ]);

            if let Some(ref c) = cursor {
                req = req.query(&[("cursor", c.as_str())]);
            }

            let resp: EventsResponse = req.send().await?.json().await?;

            for event in &resp.events {
                let title_match = event.title.to_lowercase().contains(&query_lower);
                for market in &event.markets {
                    if title_match || market.title.to_lowercase().contains(&query_lower) {
                        let probability = market
                            .last_price_dollars
                            .parse::<f64>()
                            .unwrap_or(0.0);
                        let volume_24h = market
                            .volume_24h_fp
                            .parse::<f64>()
                            .unwrap_or(0.0);
                        items.push(MarketItem {
                            title: market.title.clone(),
                            probability,
                            volume_24h,
                        });
                    }
                }
            }

            if resp.cursor.is_empty() {
                break;
            }
            cursor = Some(resp.cursor);
        }

        Ok(items)
    }
}

#[derive(Deserialize)]
struct EventsResponse {
    #[serde(default)]
    cursor: String,
    events: Vec<KalshiEvent>,
}

#[derive(Deserialize)]
struct KalshiEvent {
    title: String,
    #[serde(default)]
    markets: Vec<KalshiMarket>,
}

#[derive(Deserialize)]
struct KalshiMarket {
    title: String,
    #[serde(default)]
    last_price_dollars: String,
    #[serde(default)]
    volume_24h_fp: String,
}
