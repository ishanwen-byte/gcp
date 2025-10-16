//! Minimal GitHub Copier - lightweight version
//! Usage: gcp <github_url> [destination]

use std::env;
use std::process;
use gcp_minimal::{download_from_github, Config};

fn print_usage() {
    eprintln!("GitHub Copier - Minimal Version");
    eprintln!("Downloads files/folders from public GitHub repositories");
    eprintln!();
    eprintln!("Usage: gcp <github_url> [destination]");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  gcp https://github.com/owner/repo/blob/main/file.txt");
    eprintln!("  gcp https://github.com/owner/repo/blob/main/file.txt downloaded_file.txt");
    eprintln!("  gcp https://github.com/owner/repo/tree/main/folder");
    eprintln!("  gcp https://github.com/owner/repo/tree/main/folder ./local_folder");
    eprintln!();
    eprintln!("Note: This version only supports public repositories");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.len() > 3 {
        print_usage();
        process::exit(1);
    }

    let github_url = &args[1];
    let destination = args.get(2).map(|s| s.as_str()).unwrap_or(".");

    // Validate URL format
    if !github_url.starts_with("https://github.com/") &&
       !github_url.starts_with("https://raw.githubusercontent.com/") {
        eprintln!("Error: Only GitHub URLs are supported");
        print_usage();
        process::exit(1);
    }

    // Configure downloader
    let config = Config {
        user_agent: "gcp-minimal/0.1.0".to_string(),
        timeout_secs: 30,
    };

    // Download
    match download_from_github(github_url, destination, Some(config)) {
        Ok(()) => {
            eprintln!("✓ Download completed successfully");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}