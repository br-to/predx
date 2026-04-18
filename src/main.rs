mod kalshi;
mod market;
mod polymarket;

use clap::{Parser, Subcommand, ValueEnum};

use crate::kalshi::Kalshi;
use crate::market::{Market, MarketItem};
use crate::polymarket::Polymarket;

#[derive(Parser)]
#[command(name = "predx", about = "Cross-search prediction markets")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, ValueEnum)]
enum SortKey {
    /// Sort by trading volume (default)
    Volume,
    /// Sort by probability (highest first)
    Prob,
}

#[derive(Subcommand)]
enum Commands {
    /// Search markets by keyword
    Search {
        /// Search query (e.g. "trump", "bitcoin")
        query: String,

        /// Max results per platform (1-100, default: 20)
        #[arg(short, long, default_value_t = 20)]
        limit: usize,

        /// Sort results by: volume, prob
        #[arg(short, long, default_value = "volume")]
        sort: SortKey,

        /// Include resolved/closed markets
        #[arg(long)]
        inactive: bool,
    },
}

const COL_WIDTH: usize = 38;
const GAP: &str = "  │  ";

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max - 3).collect();
        format!("{truncated}...")
    }
}

fn format_item(item: &MarketItem) -> String {
    format!(
        "{:<title_w$} {:>5.1}%  {:>7.1}k",
        truncate(&item.title, COL_WIDTH - 16),
        item.probability * 100.0,
        item.volume / 1000.0,
        title_w = COL_WIDTH - 16,
    )
}

fn format_empty() -> String {
    " ".repeat(COL_WIDTH)
}

fn print_side_by_side(
    left_name: &str,
    left_items: &[MarketItem],
    right_name: &str,
    right_items: &[MarketItem],
    limit: usize,
) {
    let left_shown = left_items.len().min(limit);
    let right_shown = right_items.len().min(limit);

    let left_header = format!(
        "{} ({}/{})",
        left_name, left_shown, left_items.len()
    );
    let right_header = format!(
        "{} ({}/{})",
        right_name, right_shown, right_items.len()
    );
    println!("\n{:<w$}{}{}", left_header, GAP, right_header, w = COL_WIDTH);
    println!("{}{}{}", "─".repeat(COL_WIDTH), GAP, "─".repeat(COL_WIDTH));

    let rows = left_shown.max(right_shown);
    for i in 0..rows {
        let left = left_items.get(i).map(format_item).unwrap_or_else(format_empty);
        let right = right_items.get(i).map(format_item).unwrap_or_else(format_empty);
        println!("{:<w$}{}{}", left, GAP, right, w = COL_WIDTH);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_short_string() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_exact_length() {
        assert_eq!(truncate("hello", 5), "hello");
    }

    #[test]
    fn truncate_long_string() {
        assert_eq!(truncate("hello world", 8), "hello...");
    }

    #[test]
    fn truncate_multibyte() {
        assert_eq!(truncate("51st state — Puerto Rico", 20), "51st state — Puer...");
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Search { query, limit, sort, inactive } => {
            let limit = limit.clamp(1, 100);

            let poly = Polymarket::new();
            let kal = Kalshi::new();

            let (mut poly_res, mut kal_res) =
                tokio::try_join!(poly.search(&query), kal.search(&query))?;

            if !inactive {
                poly_res.retain(|item| item.active);
                kal_res.retain(|item| item.active);
            }

            let comparator: fn(&MarketItem, &MarketItem) -> std::cmp::Ordering = match sort {
                SortKey::Volume => |a, b| b.volume.partial_cmp(&a.volume).unwrap_or(std::cmp::Ordering::Equal),
                SortKey::Prob => |a, b| b.probability.partial_cmp(&a.probability).unwrap_or(std::cmp::Ordering::Equal),
            };
            poly_res.sort_by(comparator);
            kal_res.sort_by(comparator);

            print_side_by_side(
                poly.name(),
                &poly_res,
                kal.name(),
                &kal_res,
                limit,
            );

            Ok(())
        }
    }
}
