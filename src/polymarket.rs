use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;

use crate::market::{Market, MarketItem};

const SEARCH_URL: &str = "https://gamma-api.polymarket.com/public-search";
const DEFAULT_LIMIT: u32 = 10;

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

    async fn search(&self, query: &str) -> Result<Vec<MarketItem>> {
        let resp: SearchResponse = self
            .client
            .get(SEARCH_URL)
            .query(&[("q", query), ("limit", &DEFAULT_LIMIT.to_string())])
            .send()
            .await?
            .json()
            .await?;

        let mut items = Vec::new();
        for event in resp.events {
            let vol_per_market = if event.markets.is_empty() {
                0.0
            } else {
                event.volume24hr / event.markets.len() as f64
            };
            for market in event.markets {
                let probability = parse_yes_price(&market.outcome_prices);
                items.push(MarketItem {
                    title: market.question,
                    probability,
                    volume_24h: vol_per_market,
                });
            }
        }
        Ok(items)
    }
}

fn parse_yes_price(raw: &str) -> f64 {
    // outcomePrices is a JSON string like "[\"0.84\", \"0.16\"]"
    serde_json::from_str::<Vec<String>>(raw)
        .ok()
        .and_then(|v| v.first()?.parse::<f64>().ok())
        .unwrap_or(0.0)
}

#[derive(Deserialize)]
struct SearchResponse {
    events: Vec<Event>,
}

#[derive(Deserialize)]
struct Event {
    #[serde(default)]
    volume24hr: f64,
    markets: Vec<PolymarketMarket>,
}

#[derive(Deserialize)]
struct PolymarketMarket {
    question: String,
    #[serde(rename = "outcomePrices")]
    outcome_prices: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_yes_price_normal() {
        assert_eq!(parse_yes_price(r#"["0.84", "0.16"]"#), 0.84);
    }

    #[test]
    fn parse_yes_price_empty_array() {
        assert_eq!(parse_yes_price("[]"), 0.0);
    }

    #[test]
    fn parse_yes_price_invalid() {
        assert_eq!(parse_yes_price("not json"), 0.0);
    }
}
