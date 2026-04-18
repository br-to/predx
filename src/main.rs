mod kalshi;
mod market;
mod polymarket;

use clap::{Parser, Subcommand};

use crate::kalshi::Kalshi;
use crate::market::Market;
use crate::polymarket::Polymarket;

#[derive(Parser)]
#[command(name = "predx", about = "Cross-search prediction markets")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search markets by keyword
    Search {
        /// Search query (e.g. "trump", "bitcoin")
        query: String,
    },
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Search { query } => {
            let markets: Vec<Box<dyn Market>> = vec![
                Box::new(Polymarket::new()),
                Box::new(Kalshi::new()),
            ];

            for m in &markets {
                let results = m.search(&query).await?;
                println!("\n{} ({} results)", m.name(), results.len());
                println!("{:<50}  {:>6}  {:>10}", "Title", "Prob", "Volume");
                println!("{}", "─".repeat(72));
                for item in &results {
                    println!(
                        "{:<50}  {:>5.1}%  {:>9.1}k",
                        truncate(&item.title, 50),
                        item.probability * 100.0,
                        item.volume / 1000.0,
                    );
                }
            }

            Ok(())
        }
    }
}
