//! Eterea CLI - Command-line interface for bookmark management

use anyhow::Result;
use eterea_core::{Database, Ingester};
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "ingest" => {
            if args.len() < 3 {
                eprintln!("Usage: eterea-cli ingest <file_path>");
                return Ok(());
            }
            let file_path = PathBuf::from(&args[2]);
            ingest_file(&file_path)?;
        }
        "search" => {
            if args.len() < 3 {
                eprintln!("Usage: eterea-cli search <query>");
                return Ok(());
            }
            let query = args[2..].join(" ");
            search_bookmarks(&query)?;
        }
        "stats" => {
            show_stats()?;
        }
        _ => {
            print_usage();
        }
    }

    Ok(())
}

fn print_usage() {
    println!("Eterea CLI - Lightning-fast Twitter bookmarks manager");
    println!();
    println!("Usage:");
    println!("  eterea-cli ingest <file_path>  - Import bookmarks from CSV/JSON");
    println!("  eterea-cli search <query>      - Search bookmarks");
    println!("  eterea-cli stats               - Show database statistics");
}

fn ingest_file(path: &PathBuf) -> Result<()> {
    println!("📥 Ingesting bookmarks from: {}", path.display());
    
    let db = Database::open_default()?;
    let ingester = Ingester::new();
    
    let start = std::time::Instant::now();
    let count = ingester.ingest_file(path, &db)?;
    let elapsed = start.elapsed();
    
    println!("✅ Imported {} bookmarks in {:.2}s", count, elapsed.as_secs_f64());
    println!("⚡ Rate: {:.0} bookmarks/second", count as f64 / elapsed.as_secs_f64());
    
    Ok(())
}

fn search_bookmarks(query: &str) -> Result<()> {
    let db = Database::open_default()?;
    
    let start = std::time::Instant::now();
    let results = db.search(query, 20)?;
    let elapsed = start.elapsed();
    
    println!("🔍 Found {} results in {:.2}ms\n", results.len(), elapsed.as_secs_f64() * 1000.0);
    
    for bookmark in results {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("👤 @{} ({})", bookmark.author_handle, bookmark.author_name);
        println!("📅 {}", bookmark.tweeted_at.format("%Y-%m-%d %H:%M"));
        println!("📝 {}", bookmark.content);
        if !bookmark.tags.is_empty() {
            println!("🏷️  {}", bookmark.tags.join(", "));
        }
        println!("🔗 {}", bookmark.tweet_url);
        println!();
    }
    
    Ok(())
}

fn show_stats() -> Result<()> {
    let db = Database::open_default()?;
    let stats = db.get_stats()?;
    
    println!("📊 Database Statistics");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Total bookmarks: {}", stats.total_bookmarks);
    println!("Total authors:   {}", stats.unique_authors);
    println!("Total tags:      {}", stats.unique_tags);
    println!("Date range:      {} to {}", 
        stats.earliest_date.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default(),
        stats.latest_date.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default()
    );
    
    println!("\n🏷️  Top Tags:");
    for (tag, count) in stats.top_tags.iter().take(10) {
        println!("  {}: {}", tag, count);
    }
    
    Ok(())
}

