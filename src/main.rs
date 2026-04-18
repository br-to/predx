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

const GAP: &str = "  │  ";

fn title_width(items: &[MarketItem], limit: usize) -> usize {
    items.iter().take(limit).map(|i| i.title.chars().count()).max().unwrap_or(0)
}

fn format_item(item: &MarketItem, title_w: usize) -> String {
    format!(
        "{:<title_w$}  {:>5.1}%  {:>7.1}k",
        item.title,
        item.probability * 100.0,
        item.volume / 1000.0,
    )
}

fn col_width(title_w: usize) -> usize {
    title_w + 16
}

fn format_empty(w: usize) -> String {
    " ".repeat(w)
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
    let left_tw = title_width(left_items, limit);
    let right_tw = title_width(right_items, limit);
    let left_cw = col_width(left_tw);
    let right_cw = col_width(right_tw);

    let left_header = format!(
        "{} ({}/{})",
        left_name, left_shown, left_items.len()
    );
    let right_header = format!(
        "{} ({}/{})",
        right_name, right_shown, right_items.len()
    );
    println!("\n{:<w$}{}{}", left_header, GAP, right_header, w = left_cw);
    println!("{}{}{}", "─".repeat(left_cw), GAP, "─".repeat(right_cw));

    let rows = left_shown.max(right_shown);
    for i in 0..rows {
        let left = left_items.get(i).map(|i| format_item(i, left_tw)).unwrap_or_else(|| format_empty(left_cw));
        let right = right_items.get(i).map(|i| format_item(i, right_tw)).unwrap_or_else(|| format_empty(right_cw));
        println!("{:<w$}{}{}", left, GAP, right, w = left_cw);
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
