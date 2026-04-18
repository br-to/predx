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
                println!("--- {} ({} results) ---", m.name(), results.len());
                for item in &results {
                    println!(
                        "  {:50} {:>5.1}%  ${:.1}k/24h",
                        item.title,
                        item.probability * 100.0,
                        item.volume_24h / 1000.0,
                    );
                }
            }

            Ok(())
        }
    }
}
