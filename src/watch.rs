use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use serde_json::json;
use tokio::time::{Instant, sleep};

use crate::kalshi::Kalshi;
use crate::market::{Market, MarketItem};
use crate::polymarket::Polymarket;

pub enum WatchTarget {
    Query { query: String, limit: usize },
    Ids(Vec<MarketId>),
}

pub struct MarketId {
    pub platform: String,
    pub id: String,
}

pub struct WatchOptions {
    pub target: WatchTarget,
    pub interval: u64,
    pub threshold: f64,
    pub duration: Option<u64>,
    pub webhook: Option<String>,
    pub log: Option<PathBuf>,
}

pub fn parse_market_id(raw: &str) -> Result<MarketId> {
    let (platform, id) = raw
        .split_once(':')
        .with_context(|| format!("invalid --market-id \"{}\" (expected platform:id)", raw))?;
    let platform = platform.trim().to_ascii_lowercase();
    let id = id.trim().to_string();
    if !matches!(platform.as_str(), "polymarket" | "kalshi") {
        anyhow::bail!("unknown platform \"{}\": use polymarket or kalshi", platform);
    }
    if id.is_empty() {
        anyhow::bail!("market id cannot be empty");
    }
    Ok(MarketId { platform, id })
}

fn append_log(path: &PathBuf, line: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("failed to open log file {}", path.display()))?;
    writeln!(file, "{}", line).with_context(|| "failed to write log line")?;
    Ok(())
}

enum WebhookFlavor {
    Discord,
    Slack,
}

fn detect_flavor(url: &str) -> WebhookFlavor {
    if url.contains("discord.com") || url.contains("discordapp.com") {
        WebhookFlavor::Discord
    } else {
        WebhookFlavor::Slack
    }
}

async fn post_webhook(client: &reqwest::Client, url: &str, message: &str) -> Result<()> {
    let body = match detect_flavor(url) {
        WebhookFlavor::Discord => json!({ "content": message }),
        WebhookFlavor::Slack => json!({ "text": message }),
    };
    let resp = client.post(url).json(&body).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("webhook returned status {}", resp.status());
    }
    Ok(())
}

pub async fn run(opts: WatchOptions) -> Result<()> {
    let poly = Polymarket::new();
    let kal = Kalshi::new();
    let http = reqwest::Client::new();

    let deadline = opts.duration.map(|m| Instant::now() + Duration::from_secs(m * 60));
    let mut prev: HashMap<String, f64> = HashMap::new();
    let mut tick = 0u64;

    match &opts.target {
        WatchTarget::Query { query, limit } => println!(
            "# watching \"{}\" — interval={}s threshold={}% limit={}",
            query, opts.interval, opts.threshold, limit,
        ),
        WatchTarget::Ids(ids) => println!(
            "# watching {} market(s) — interval={}s threshold={}%",
            ids.len(),
            opts.interval,
            opts.threshold,
        ),
    }
    if let Some(d) = opts.duration {
        println!("# stop after {} minute(s)", d);
    }
    if opts.webhook.is_some() {
        println!("# webhook notifications enabled");
    }
    if let Some(p) = &opts.log {
        println!("# logging to {}", p.display());
    }
    println!("# press Ctrl+C to stop");

    loop {
        let snapshot = fetch_snapshot(&poly, &kal, &opts.target).await;
        match snapshot {
            Ok(items) => {
                let now = chrono_now();
                for item in &items {
                    let key = format!("{}:{}", item.platform, item.id);
                    let curr = item.probability * 100.0;
                    if let Some(&prev_p) = prev.get(&key) {
                        let diff = curr - prev_p;
                        if diff.abs() >= opts.threshold {
                            let sign = if diff >= 0.0 { "+" } else { "" };
                            let line = format!(
                                "[{}] [{}] {}  {:.1}% → {:.1}% ({}{:.1}pp)",
                                now, item.platform, item.title, prev_p, curr, sign, diff,
                            );
                            println!("{}", line);
                            if let Some(url) = &opts.webhook
                                && let Err(e) = post_webhook(&http, url, &line).await
                            {
                                eprintln!("[warn] webhook failed: {}", e);
                            }
                            if let Some(path) = &opts.log
                                && let Err(e) = append_log(path, &line)
                            {
                                eprintln!("[warn] log write failed: {}", e);
                            }
                        }
                    } else if tick == 0 {
                        let baseline = format!(
                            "[{}] [{}] {}  baseline {:.1}%",
                            now, item.platform, item.title, curr,
                        );
                        println!("{}", baseline);
                        if let Some(path) = &opts.log
                            && let Err(e) = append_log(path, &baseline)
                        {
                            eprintln!("[warn] log write failed: {}", e);
                        }
                    }
                    prev.insert(key, curr);
                }
            }
            Err(e) => {
                eprintln!("[warn] fetch failed: {}", e);
            }
        }

        tick += 1;

        if let Some(dl) = deadline
            && Instant::now() >= dl
        {
            println!("# duration reached, stopping");
            return Ok(());
        }

        let wait = Duration::from_secs(opts.interval);
        tokio::select! {
            _ = sleep(wait) => {}
            _ = tokio::signal::ctrl_c() => {
                println!("\n# interrupted, stopping");
                return Ok(());
            }
        }
    }
}

async fn fetch_snapshot(
    poly: &Polymarket,
    kal: &Kalshi,
    target: &WatchTarget,
) -> Result<Vec<MarketItem>> {
    match target {
        WatchTarget::Query { query, limit } => {
            let (mut poly_res, mut kal_res) =
                tokio::try_join!(poly.search(query), kal.search(query))?;
            poly_res.retain(|item| item.active);
            kal_res.retain(|item| item.active);

            let cmp = |a: &MarketItem, b: &MarketItem| {
                b.volume
                    .partial_cmp(&a.volume)
                    .unwrap_or(std::cmp::Ordering::Equal)
            };
            poly_res.sort_by(cmp);
            kal_res.sort_by(cmp);

            let mut items: Vec<MarketItem> = poly_res.into_iter().take(*limit).collect();
            items.extend(kal_res.into_iter().take(*limit));
            Ok(items)
        }
        WatchTarget::Ids(ids) => {
            let mut items = Vec::with_capacity(ids.len());
            for mid in ids {
                let result = match mid.platform.as_str() {
                    "polymarket" => poly.get_by_id(&mid.id).await,
                    "kalshi" => kal.get_by_id(&mid.id).await,
                    other => anyhow::bail!("unknown platform: {}", other),
                };
                match result {
                    Ok(item) => items.push(item),
                    Err(e) => eprintln!("[warn] fetch {}:{} failed: {}", mid.platform, mid.id, e),
                }
            }
            Ok(items)
        }
    }
}

fn chrono_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}Z", h, m, s)
}
