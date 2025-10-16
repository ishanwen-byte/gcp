//! Minimal GitHub Copier - lightweight version
//! Usage: gcp <github_url> [destination]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use std::env;
use std::process;
use gcp::download_from_github;

fn print_usage() {
    eprintln!("GitHub Copier - Minimal Version");
    eprintln!("Downloads files/folders from public GitHub repositories");
    eprintln!();
    eprintln!("Usage: gcp [OPTIONS] <github_url> [destination]");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  -h, --help     Print help information");
    eprintln!();
    eprintln!("ARGUMENTS:");
    eprintln!("  <github_url>    GitHub URL to download from");
    eprintln!("  <destination>   Local destination path (optional)");
    eprintln!();
    eprintln!("EXAMPLES:");
    eprintln!("  gcp https://github.com/owner/repo/blob/main/file.txt");
    eprintln!("  gcp https://github.com/owner/repo/blob/main/file.txt downloaded_file.txt");
    eprintln!("  gcp https://github.com/owner/repo/tree/main/folder");
    eprintln!("  gcp https://github.com/owner/repo/tree/main/folder ./local_folder");
    eprintln!();
    eprintln!("NOTES:");
    eprintln!("  Downloads with original filename by default when no destination is specified");
    eprintln!("  Only supports public repositories in this minimal version");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Handle help flag
    if args.len() == 2 && (args[1] == "--help" || args[1] == "-h") {
        print_usage();
        process::exit(0);
    }

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

    // Download
    match download_from_github(github_url, destination) {
        Ok(()) => {
            eprintln!("✓ Download completed successfully");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}