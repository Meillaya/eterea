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
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, PartialEq, Eq)]
enum Command {
    ImportAll,
    ImportFile(PathBuf),
    Help,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    let parsed = match parse_args(&args[1..]) {
        Ok(parsed) => parsed,
        Err(error) => {
            eprintln!("Error: {error}");
            print_usage();
            return;
        }
    };

    match parsed.command {
        Command::ImportAll => import_all(parsed.dry_run),
        Command::ImportFile(path) => import_file(&path, parsed.dry_run),
        Command::Help => print_usage(),
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ParsedArgs {
    dry_run: bool,
    command: Command,
}

fn parse_args(args: &[String]) -> Result<ParsedArgs, String> {
    let mut dry_run = false;
    let mut import_all = false;
    let mut file_path: Option<PathBuf> = None;
    let mut pending_path_flag: Option<&str> = None;

    for arg in args {
        if let Some(flag) = pending_path_flag.take() {
            file_path = Some(PathBuf::from(arg));
            if flag == "--all" {
                return Err("--all does not take a path".to_string());
            }
            continue;
        }

        match arg.as_str() {
            "--dry-run" => dry_run = true,
            "--help" | "-h" => {
                return Ok(ParsedArgs {
                    dry_run,
                    command: Command::Help,
                });
            }
            "--all" => import_all = true,
            "--legacy" | "--new" | "--json" => pending_path_flag = Some(arg.as_str()),
            value if value.starts_with("--") => {
                return Err(format!("Unknown option: {value}"));
            }
            value => {
                if file_path.is_some() {
                    return Err(format!("Unexpected extra argument: {value}"));
                }
                file_path = Some(PathBuf::from(value));
            }
        }
    }

    if let Some(flag) = pending_path_flag {
        return Err(format!("{flag} requires a file path"));
    }

    if import_all && file_path.is_some() {
        return Err("Cannot combine --all with an explicit file path".to_string());
    }

    let command = if import_all {
        Command::ImportAll
    } else if let Some(path) = file_path {
        Command::ImportFile(path)
    } else {
        return Err("No input file or --all provided".to_string());
    };

    Ok(ParsedArgs { dry_run, command })
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
    println!("    migrate src/legacy/new_bookmarks.json --dry-run");
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
            ext == "csv" || ext == "json" || ext == "js"
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
    let mut total_skipped = 0;

    for file in files {
        match import_single_file(&file, dry_run) {
            Ok(summary) => {
                total_imported += summary.imported;
                total_skipped += summary.skipped;
            }
            Err(e) => {
                eprintln!("  ❌ Error: {}", e);
            }
        }
    }

    let total_elapsed = total_start.elapsed();
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!(
        "✅ Total: {} bookmarks imported in {:.2}s",
        total_imported,
        total_elapsed.as_secs_f64()
    );
    if total_skipped > 0 {
        println!("   ↷ Skipped {} duplicates/already-imported bookmarks", total_skipped);
    }

    if dry_run {
        println!("   (dry run - no data was saved)");
    }
}

fn import_file(path: &Path, dry_run: bool) {
    match import_single_file(path, dry_run) {
        Ok(summary) => {
            if dry_run {
                println!(
                    "✅ Would import {} bookmarks (dry run)",
                    summary.imported
                );
            }
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ImportSummary {
    imported: usize,
    skipped: usize,
}

fn import_single_file(path: &Path, dry_run: bool) -> Result<ImportSummary, String> {
    println!("📥 Processing: {}", path.display());

    let start = Instant::now();
    let ingester = Ingester::new();
    let parsed_bookmarks = ingester.parse_file(path).map_err(|e| e.to_string())?;
    let parsed_count = parsed_bookmarks.len();

    if dry_run {
        let elapsed = start.elapsed();
        println!(
            "  ✓ Parsed {} bookmarks in {:.2}s (dry run)",
            parsed_count,
            elapsed.as_secs_f64()
        );

        return Ok(ImportSummary {
            imported: parsed_count,
            skipped: 0,
        });
    }

    let db = Database::open_default().map_err(|e| e.to_string())?;
    let imported = db
        .insert_bookmarks(&parsed_bookmarks)
        .map_err(|e| e.to_string())?;
    let skipped = parsed_count.saturating_sub(imported);

    let elapsed = start.elapsed();
    let rate = if elapsed.as_secs_f64() > 0.0 {
        imported as f64 / elapsed.as_secs_f64()
    } else {
        imported as f64
    };

    if skipped == 0 {
        println!(
            "  ✓ Imported {} bookmarks in {:.2}s ({:.0}/sec)",
            imported,
            elapsed.as_secs_f64(),
            rate
        );
    } else if imported == 0 {
        println!(
            "  ✓ Imported 0 new bookmarks in {:.2}s (all {} were already in the database)",
            elapsed.as_secs_f64(),
            skipped
        );
    } else {
        println!(
            "  ✓ Imported {} bookmarks in {:.2}s ({:.0}/sec), skipped {} duplicates",
            imported,
            elapsed.as_secs_f64(),
            rate,
            skipped
        );
    }

    Ok(ImportSummary { imported, skipped })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn parses_dry_run_before_file() {
        let parsed = parse_args(&args(&["--dry-run", "bookmarks.json"])).unwrap();
        assert_eq!(
            parsed,
            ParsedArgs {
                dry_run: true,
                command: Command::ImportFile(PathBuf::from("bookmarks.json"))
            }
        );
    }

    #[test]
    fn parses_dry_run_after_file() {
        let parsed = parse_args(&args(&["bookmarks.json", "--dry-run"])).unwrap();
        assert_eq!(
            parsed,
            ParsedArgs {
                dry_run: true,
                command: Command::ImportFile(PathBuf::from("bookmarks.json"))
            }
        );
    }

    #[test]
    fn parses_all_with_dry_run() {
        let parsed = parse_args(&args(&["--all", "--dry-run"])).unwrap();
        assert_eq!(
            parsed,
            ParsedArgs {
                dry_run: true,
                command: Command::ImportAll
            }
        );
    }

    #[test]
    fn rejects_all_with_file() {
        let error = parse_args(&args(&["--all", "bookmarks.json"])).unwrap_err();
        assert!(error.contains("Cannot combine --all"));
    }
}
