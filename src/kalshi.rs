use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;

use crate::market::{Market, MarketItem};

const SEARCH_URL: &str = "https://api.elections.kalshi.com/v1/search/series";
const MARKET_URL: &str = "https://api.elections.kalshi.com/trade-api/v2/markets";
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

    async fn get_by_id(&self, id: &str) -> Result<MarketItem> {
        let url = format!("{}/{}", MARKET_URL, id);
        let resp: V2MarketResponse = self.client.get(&url).send().await?.json().await?;
        let m = resp.market;
        let probability = m.last_price_dollars.parse::<f64>().unwrap_or(0.0);
        let volume = m.volume_fp.parse::<f64>().unwrap_or(0.0);
        let title = if m.yes_sub_title.is_empty() {
            m.title
        } else {
            format!("{} — {}", m.title, m.yes_sub_title)
        };
        Ok(MarketItem {
            id: m.ticker,
            platform: "kalshi",
            title,
            probability,
            volume,
            active: m.status == "active",
        })
    }
}

#[derive(Deserialize)]
struct V2MarketResponse {
    market: V2Market,
}

#[derive(Deserialize)]
struct V2Market {
    #[serde(default)]
    ticker: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    yes_sub_title: String,
    #[serde(default)]
    last_price_dollars: String,
    #[serde(default)]
    volume_fp: String,
    #[serde(default)]
    status: String,
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
