use clap::Parser;
use std::path::PathBuf;
use tracing::{info, error, debug};

#[derive(Parser)]
#[command(name = "gcp")]
#[command(about = "Copy files/folders from GitHub repositories")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    /// GitHub URL to copy from (file or folder)
    /// Examples:
    ///   https://github.com/owner/repo/blob/main/path/to/file.txt
    ///   https://github.com/owner/repo/tree/main/folder-name
    ///   https://raw.githubusercontent.com/owner/repo/main/file.txt
    #[arg(value_parser = validate_github_url)]
    source: String,

    /// Local destination path
    #[arg(value_parser = validate_local_path)]
    destination: Option<PathBuf>,

    /// GitHub authentication token (or use GITHUB_TOKEN env var)
    #[arg(long, short = 't')]
    auth_token: Option<String>,

    /// Enable verbose output
    #[arg(long, short = 'v')]
    verbose: bool,

    /// Suppress non-error output
    #[arg(long, short = 'q')]
    quiet: bool,

    /// Show what would be copied without actually copying
    #[arg(long)]
    dry_run: bool,

    /// Show progress bar
    #[arg(long)]
    progress: bool,

    /// Overwrite existing files (default: auto-rename)
    #[arg(long, short = 'f')]
    force: bool,

    /// Preserve original file modification times
    #[arg(long)]
    preserve_timestamps: bool,

    /// Exclude files matching pattern (glob)
    #[arg(long)]
    exclude: Vec<String>,

    /// Include only files matching pattern (glob)
    #[arg(long)]
    include: Vec<String>,

    /// Maximum concurrent downloads (default: 10)
    #[arg(long, default_value = "10")]
    max_concurrent: usize,

    /// Request timeout in seconds (default: 30)
    #[arg(long, default_value = "30")]
    timeout: u64,

    /// Maximum retry attempts (default: 3)
    #[arg(long, default_value = "3")]
    retry: u32,

    /// Cache directory for metadata
    #[arg(long)]
    cache_dir: Option<PathBuf>,

    /// Disable caching
    #[arg(long)]
    no_cache: bool,
}

fn validate_github_url(url: &str) -> Result<String, String> {
    // Basic URL validation - more comprehensive validation will be done in GitHubUrl::parse
    if url.is_empty() {
        return Err("URL cannot be empty".to_string());
    }

    if !url.starts_with("https://github.com/") && !url.starts_with("https://raw.githubusercontent.com/") {
        return Err("URL must start with https://github.com/ or https://raw.githubusercontent.com/".to_string());
    }

    Ok(url.to_string())
}

fn validate_local_path(path: &str) -> Result<PathBuf, String> {
    let path_buf = PathBuf::from(path);

    // Check for obviously invalid characters (platform-specific)
    #[cfg(windows)]
    {
        if path.contains("<") || path.contains(">") || path.contains("\"") || path.contains("|") || path.contains("?") || path.contains("*") {
            return Err("Path contains invalid characters".to_string());
        }
    }

    #[cfg(not(windows))]
    {
        if path.contains('\0') {
            return Err("Path contains null byte".to_string());
        }
    }

    Ok(path_buf)
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose {
        tracing::Level::DEBUG
    } else if cli.quiet {
        tracing::Level::ERROR
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    // Validate input arguments
    let destination = cli.destination.unwrap_or_else(|| PathBuf::from("."));

    info!("Starting GitHub Copy Tool");
    info!("Source: {}", cli.source);
    info!("Destination: {}", destination.display());

    // Parse and validate the GitHub URL
    let github_url = match gcp::github::GitHubUrl::parse(&cli.source) {
        Ok(url) => {
            debug!("Parsed GitHub URL: {:?}", url);
            url
        }
        Err(e) => {
            error!("Failed to parse GitHub URL: {}", e);
            std::process::exit(1);
        }
    };

    // Handle authentication
    let auth = if let Some(token) = cli.auth_token {
        Some(gcp::github::Authentication {
            token,
            scopes: vec![],
            expires_at: None,
            source: gcp::github::AuthSource::CommandLine,
        })
    } else {
        // Try environment variable
        match gcp::github::Authentication::from_env() {
            Ok(Some(auth)) => Some(auth),
            Ok(None) => None,
            Err(e) => {
                error!("Authentication error: {}", e);
                std::process::exit(1);
            }
        }
    };

    // Create configuration
    let config = gcp::Config {
        github: gcp::GitHubConfig {
            api_url: "https://api.github.com".to_string(),
            max_concurrent_requests: cli.max_concurrent,
            retry_attempts: cli.retry,
            rate_limit_buffer: 5,
        },
        download: gcp::DownloadConfig {
            chunk_size: 1024 * 1024, // 1MB
            max_file_size: 100 * 1024 * 1024, // 100MB
            timeout_seconds: cli.timeout,
        },
        filesystem: gcp::FilesystemConfig {
            default_permissions: Some(0o644),
            preserve_timestamps: cli.preserve_timestamps,
            create_intermediate_dirs: true,
        },
    };

    // Create GitHub client
    let github_client = match gcp::github::GitHubClient::new(config.clone(), auth).await {
        Ok(client) => std::sync::Arc::new(client),
        Err(e) => {
            error!("Failed to create GitHub client: {}", e);
            std::process::exit(1);
        }
    };

    // Determine final destination based on GitHub URL type
    let final_destination = match github_url.url_type {
        gcp::github::UrlType::File => {
            if destination.is_dir() {
                // If destination is a directory, use the original filename
                if let Some(path) = &github_url.path {
                    let filename = std::path::Path::new(path)
                        .file_name()
                        .unwrap_or_default();
                    destination.join(filename)
                } else {
                    destination
                }
            } else {
                destination
            }
        }
        gcp::github::UrlType::Folder => {
            destination
        }
        gcp::github::UrlType::Repository => {
            error!("Repository URLs are not supported. Use file or folder URLs only.");
            std::process::exit(1);
        }
    };

    info!("Final destination: {}", final_destination.display());

    if cli.dry_run {
        info!("DRY RUN: Would download from {}", cli.source);
        info!("DRY RUN: Would save to {}", final_destination.display());
        return;
    }

    // Create progress reporter if needed
    let progress = if cli.progress || github_url.url_type == gcp::github::UrlType::Folder {
        Some(std::sync::Arc::new(gcp::downloader::ProgressReporter::new_spinner("Downloading...")))
    } else {
        None
    };

    // Execute download based on URL type
    let result = match github_url.url_type {
        gcp::github::UrlType::File => {
            info!("Downloading single file");
            let file_downloader = gcp::downloader::FileDownloader::new(github_client.clone())
                .with_progress(progress.unwrap_or_else(|| std::sync::Arc::new(gcp::downloader::ProgressReporter::new(1))));

            file_downloader.download_file(&github_url, &final_destination, cli.force).await
        }
        gcp::github::UrlType::Folder => {
            info!("Downloading folder");
            let folder_downloader = gcp::downloader::FolderDownloader::new(github_client.clone())
                .with_progress(progress.unwrap_or_else(|| std::sync::Arc::new(gcp::downloader::ProgressReporter::new_spinner("Downloading folder..."))));

            match folder_downloader.download_folder(&github_url, &final_destination, cli.force).await {
                Ok(count) => {
                    info!("Downloaded {} files", count);
                    Ok(final_destination)
                }
                Err(e) => Err(e)
            }
        }
        gcp::github::UrlType::Repository => {
            unreachable!() // Handled above
        }
    };

    match result {
        Ok(path) => {
            if !cli.quiet {
                println!("✓ Successfully copied to {}", path.display());
            }
        }
        Err(e) => {
            error!("Download failed: {}", e);
            std::process::exit(1);
        }
    }

    info!("GitHub Copy Tool finished successfully");
}
