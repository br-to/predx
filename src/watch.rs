use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;
use tokio::time::{Instant, sleep};

use crate::kalshi::Kalshi;
use crate::market::{Market, MarketItem};
use crate::polymarket::Polymarket;

pub struct WatchOptions {
    pub query: String,
    pub interval: u64,
    pub threshold: f64,
    pub limit: usize,
    pub duration: Option<u64>,
}

pub async fn run(opts: WatchOptions) -> Result<()> {
    let poly = Polymarket::new();
    let kal = Kalshi::new();

    let deadline = opts.duration.map(|m| Instant::now() + Duration::from_secs(m * 60));
    let mut prev: HashMap<String, f64> = HashMap::new();
    let mut tick = 0u64;

    println!(
        "# watching \"{}\" — interval={}s threshold={}% limit={}",
        opts.query, opts.interval, opts.threshold, opts.limit,
    );
    if let Some(d) = opts.duration {
        println!("# stop after {} minute(s)", d);
    }
    println!("# press Ctrl+C to stop");

    loop {
        let snapshot = fetch_snapshot(&poly, &kal, &opts.query, opts.limit).await;
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
                            println!(
                                "[{}] [{}] {}  {:.1}% → {:.1}% ({}{:.1}pp)",
                                now, item.platform, item.title, prev_p, curr, sign, diff,
                            );
                        }
                    } else if tick == 0 {
                        println!(
                            "[{}] [{}] {}  baseline {:.1}%",
                            now, item.platform, item.title, curr,
                        );
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
    query: &str,
    limit: usize,
) -> Result<Vec<MarketItem>> {
    let (mut poly_res, mut kal_res) = tokio::try_join!(poly.search(query), kal.search(query))?;
    poly_res.retain(|item| item.active);
    kal_res.retain(|item| item.active);

    let cmp = |a: &MarketItem, b: &MarketItem| {
        b.volume
            .partial_cmp(&a.volume)
            .unwrap_or(std::cmp::Ordering::Equal)
    };
    poly_res.sort_by(cmp);
    kal_res.sort_by(cmp);

    let mut items: Vec<MarketItem> = poly_res.into_iter().take(limit).collect();
    items.extend(kal_res.into_iter().take(limit));
    Ok(items)
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
