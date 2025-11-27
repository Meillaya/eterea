//! Migration script to import existing bookmarks
//!
//! Run with: cargo run --bin migrate -- [options]
//!
//! Options:
//!   --legacy <path>   Import from legacy CSV format
//!   --new <path>      Import from new CSV format  
//!   --json <path>     Import from JSON format
//!   --all             Import all files from src/legacy/
//!   --dry-run         Parse but don't save to database

use eterea_core::{Database, Ingester};
use std::env;
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return;
    }
    
    let dry_run = args.contains(&"--dry-run".to_string());
    
    match args[1].as_str() {
        "--all" => import_all(dry_run),
        "--legacy" => {
            if let Some(path) = args.get(2) {
                import_file(path, dry_run);
            } else {
                eprintln!("Error: --legacy requires a file path");
            }
        }
        "--new" => {
            if let Some(path) = args.get(2) {
                import_file(path, dry_run);
            } else {
                eprintln!("Error: --new requires a file path");
            }
        }
        "--json" => {
            if let Some(path) = args.get(2) {
                import_file(path, dry_run);
            } else {
                eprintln!("Error: --json requires a file path");
            }
        }
        "--help" | "-h" => print_usage(),
        _ => {
            // Assume it's a file path
            import_file(&args[1], dry_run);
        }
    }
}

fn print_usage() {
    println!("Eterea Migration Tool");
    println!("=====================");
    println!();
    println!("Import your Twitter bookmarks into the Eterea database.");
    println!();
    println!("USAGE:");
    println!("    migrate [OPTIONS] [FILE]");
    println!();
    println!("OPTIONS:");
    println!("    --all           Import all files from src/legacy/");
    println!("    --legacy PATH   Import from legacy CSV format (Dewey)");
    println!("    --new PATH      Import from new CSV format (Twitter/X)");
    println!("    --json PATH     Import from JSON format");
    println!("    --dry-run       Parse files but don't save to database");
    println!("    --help, -h      Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    migrate --all");
    println!("    migrate --legacy bookmarks.csv");
    println!("    migrate src/legacy/new_bookmarks.csv");
    println!("    migrate --dry-run --all");
}

fn import_all(dry_run: bool) {
    println!("📂 Importing all bookmark files from src/legacy/");
    println!();
    
    let legacy_dir = PathBuf::from("src/legacy");
    if !legacy_dir.exists() {
        eprintln!("Error: src/legacy directory not found");
        return;
    }
    
    let files: Vec<PathBuf> = std::fs::read_dir(&legacy_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
            ext == "csv" || ext == "json"
        })
        .collect();
    
    if files.is_empty() {
        println!("No CSV or JSON files found in src/legacy/");
        return;
    }
    
    println!("Found {} file(s):", files.len());
    for f in &files {
        println!("  - {}", f.display());
    }
    println!();
    
    let total_start = Instant::now();
    let mut total_imported = 0;
    
    for file in files {
        match import_single_file(&file, dry_run) {
            Ok(count) => {
                total_imported += count;
            }
            Err(e) => {
                eprintln!("  ❌ Error: {}", e);
            }
        }
    }
    
    let total_elapsed = total_start.elapsed();
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Total: {} bookmarks imported in {:.2}s", 
        total_imported, total_elapsed.as_secs_f64());
    
    if dry_run {
        println!("   (dry run - no data was saved)");
    }
}

fn import_file(path: &str, dry_run: bool) {
    let path = PathBuf::from(path);
    match import_single_file(&path, dry_run) {
        Ok(count) => {
            if dry_run {
                println!("✅ Would import {} bookmarks (dry run)", count);
            }
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
        }
    }
}

fn import_single_file(path: &PathBuf, dry_run: bool) -> Result<usize, String> {
    println!("📥 Processing: {}", path.display());
    
    let start = Instant::now();
    
    if dry_run {
        // Just parse and count
        let ingester = Ingester::new();
        // For dry run, we'd need to add a parse-only method
        // For now, just create an in-memory DB
        let db = Database::open_memory().map_err(|e| e.to_string())?;
        let count = ingester.ingest_file(path, &db).map_err(|e| e.to_string())?;
        
        let elapsed = start.elapsed();
        println!("  ✓ Parsed {} bookmarks in {:.2}s (dry run)", 
            count, elapsed.as_secs_f64());
        
        return Ok(count);
    }
    
    let db = Database::open_default().map_err(|e| e.to_string())?;
    let ingester = Ingester::new();
    
    let count = ingester.ingest_file(path, &db).map_err(|e| e.to_string())?;
    
    let elapsed = start.elapsed();
    let rate = count as f64 / elapsed.as_secs_f64();
    
    println!("  ✓ Imported {} bookmarks in {:.2}s ({:.0}/sec)", 
        count, elapsed.as_secs_f64(), rate);
    
    Ok(count)
}

