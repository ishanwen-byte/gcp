//! GitHub client with integrated HTTPS functionality

use std::io::{Read, Write, BufRead, BufReader};
use std::net::TcpStream;
use crate::error::{GcpError, GcpResult};
use crate::github::{GitHubUrl, UrlType};
use crate::base64::Base64Decoder;

/// GitHub client with integrated HTTPS
pub struct GitHubClient {
    tls_connector: native_tls::TlsConnector,
}

impl GitHubClient {
    /// Create new GitHub client
    pub fn new() -> GcpResult<Self> {
        let tls_connector = native_tls::TlsConnector::new()
            .map_err(|e| GcpError::NetworkError(format!("Failed to create TLS connector: {}", e)))?;

        Ok(Self { tls_connector })
    }

    /// Download content from GitHub URL
    pub fn download(&self, url: &GitHubUrl, destination: &str) -> GcpResult<()> {
        match url.url_type {
            UrlType::File => self.download_file(url, destination),
            UrlType::Folder => self.download_folder(url, destination),
            UrlType::Repository => Err(GcpError::UnsupportedOperation(
                "Repository downloads not supported in minimal version".to_string()
            )),
        }
    }

    /// Download a single file
    fn download_file(&self, url: &GitHubUrl, destination: &str) -> GcpResult<()> {
        // Get file content via GitHub API
        let api_url = url.api_url();
        let (host, path) = self.parse_url(&api_url)?;
        let response = self.http_get(&host, &path)?;

        // Parse response and extract content
        let response_text = std::str::from_utf8(&response)
            .map_err(|e| GcpError::ParseError(format!("Invalid UTF-8 in response: {}", e)))?;
        let file_info = GitHubFile::from_json(response_text)?;

        // Write file content
        match file_info.content {
            Some(content) if file_info.encoding.as_deref() == Some("base64") => {
                let clean_content = content.trim().replace("\\n", "").replace("\\r", "").replace('\n', "").replace('\r', "");
                let decoded = Base64Decoder::decode(&clean_content)
                    .map_err(|e| GcpError::ParseError(format!("Base64 decode error: {}", e)))?;
                let content_str = String::from_utf8(decoded)
                    .map_err(|e| GcpError::ParseError(format!("UTF-8 decode error: {}", e)))?;
                std::fs::write(destination, content_str)?;
            }
            Some(content) => {
                // Content is not base64 encoded
                std::fs::write(destination, content)?;
            }
            None if file_info.download_url.is_some() => {
                // Use download URL as fallback
                let download_url = file_info.download_url.unwrap();
                let (host, path) = self.parse_url(&download_url)?;
                let content = self.http_get(&host, &path)?;
                std::fs::write(destination, content)?;
            }
            None => {
                return Err(GcpError::NetworkError("No file content available".to_string()));
            }
        }

        Ok(())
    }

    /// Download a folder recursively
    fn download_folder(&self, url: &GitHubUrl, destination: &str) -> GcpResult<()> {
        // Create destination directory
        std::fs::create_dir_all(destination)?;

        // Get folder contents via GitHub API
        let api_url = url.api_url();
        let (host, path) = self.parse_url(&api_url)?;
        let response = self.http_get(&host, &path)?;
        let response_text = std::str::from_utf8(&response)
            .map_err(|e| GcpError::ParseError(format!("Invalid UTF-8 in response: {}", e)))?;
        let files = parse_github_file_array(response_text)?;

        let mut downloaded_count = 0;

        for file in files {
            let file_path = std::path::Path::new(destination).join(&file.name);

            if file.file_type == "file" {
                if let Some(download_url) = file.download_url {
                    let (host, path) = self.parse_url(&download_url)?;
                    let content = self.http_get(&host, &path)?;
                    std::fs::write(&file_path, content)?;
                    downloaded_count += 1;
                }
            } else if file.file_type == "dir" {
                // Recursively download subdirectory
                let sub_url = GitHubUrl {
                    owner: url.owner.clone(),
                    repo: url.repo.clone(),
                    path: Some(file.path.clone()),
                    ref_: url.ref_.clone(),
                    url_type: UrlType::Folder,
                    raw_url: String::new(),
                };

                if let Err(e) = self.download_folder(&sub_url, file_path.to_str().unwrap_or(&file.name)) {
                    eprintln!("Warning: Failed to download folder {}: {}", file.name, e);
                }
            }
        }

        eprintln!("Downloaded {} files to {}", downloaded_count, destination);
        Ok(())
    }

    /// Parse URL into host and path
    fn parse_url(&self, url: &str) -> GcpResult<(String, String)> {
        if !url.starts_with("https://") {
            return Err(GcpError::InvalidUrl("Only HTTPS URLs are supported".to_string()));
        }

        let remaining = &url[8..]; // Remove "https://"
        if let Some(slash_pos) = remaining.find('/') {
            let host = remaining[..slash_pos].to_string();
            let path = if slash_pos < remaining.len() - 1 {
                remaining[slash_pos..].to_string()
            } else {
                "/".to_string()
            };
            Ok((host, path))
        } else {
            Err(GcpError::InvalidUrl("Invalid URL format".to_string()))
        }
    }

    /// Make HTTPS GET request
    fn http_get(&self, host: &str, path: &str) -> GcpResult<Vec<u8>> {
        // Connect to TCP
        let tcp_stream = TcpStream::connect((host, 443))
            .map_err(|e| GcpError::NetworkError(format!("TCP connection failed: {}", e)))?;

        // Perform TLS handshake
        let mut tls_stream = self.tls_connector
            .connect(host, tcp_stream)
            .map_err(|e| GcpError::NetworkError(format!("TLS handshake failed: {}", e)))?;

        // Send HTTP request
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: gcp/0.1.0\r\nConnection: close\r\nAccept: */*\r\n\r\n",
            path, host
        );

        tls_stream.write_all(request.as_bytes())
            .map_err(|e| GcpError::NetworkError(format!("Failed to send request: {}", e)))?;

        // Read and parse HTTP response
        self.read_http_response(&mut tls_stream)
    }

    /// Read and parse HTTP response
    fn read_http_response(&self, stream: &mut impl Read) -> GcpResult<Vec<u8>> {
        let mut reader = BufReader::new(stream);

        // Read status line
        let mut status_line = String::new();
        reader.read_line(&mut status_line)
            .map_err(|e| GcpError::NetworkError(format!("Failed to read status: {}", e)))?;

        // Check status code
        if !status_line.starts_with("HTTP/1.1 2") && !status_line.starts_with("HTTP/1.0 2") {
            return Err(GcpError::NetworkError(format!(
                "HTTP request failed: {}",
                status_line.trim()
            )));
        }

        // Skip headers and read body
        let mut content_length: Option<usize> = None;
        let mut chunked = false;

        loop {
            let mut line = String::new();
            reader.read_line(&mut line)
                .map_err(|e| GcpError::NetworkError(format!("Failed to read header: {}", e)))?;

            if line.trim().is_empty() {
                break;
            }

            if line.to_lowercase().starts_with("content-length:") {
                if let Some(len_str) = line.split(':').nth(1) {
                    if let Ok(len) = len_str.trim().parse::<usize>() {
                        content_length = Some(len);
                    }
                }
            }

            if line.to_lowercase().starts_with("transfer-encoding:") && line.to_lowercase().contains("chunked") {
                chunked = true;
            }
        }

        // Read body based on transfer encoding
        if let Some(length) = content_length {
            let mut body = vec![0u8; length];
            reader.read_exact(&mut body)
                .map_err(|e| GcpError::NetworkError(format!("Failed to read body: {}", e)))?;
            Ok(body)
        } else if chunked {
            self.read_chunked_body(&mut reader)
        } else {
            let mut body = Vec::new();
            reader.read_to_end(&mut body)
                .map_err(|e| GcpError::NetworkError(format!("Failed to read body: {}", e)))?;
            Ok(body)
        }
    }

    /// Read chunked transfer encoding body
    fn read_chunked_body(&self, reader: &mut impl BufRead) -> GcpResult<Vec<u8>> {
        let mut body = Vec::new();

        loop {
            let mut chunk_size_line = String::new();
            reader.read_line(&mut chunk_size_line)
                .map_err(|e| GcpError::NetworkError(format!("Failed to read chunk size: {}", e)))?;

            let chunk_size = usize::from_str_radix(chunk_size_line.trim(), 16)
                .map_err(|_| GcpError::ParseError("Invalid chunk size".to_string()))?;

            if chunk_size == 0 {
                break;
            }

            let mut chunk = vec![0u8; chunk_size];
            reader.read_exact(&mut chunk)
                .map_err(|e| GcpError::NetworkError(format!("Failed to read chunk: {}", e)))?;
            body.extend_from_slice(&chunk);

            let mut crlf = [0u8; 2];
            reader.read_exact(&mut crlf)
                .map_err(|e| GcpError::NetworkError(format!("Failed to read CRLF: {}", e)))?;
        }

        // Skip trailer headers
        loop {
            let mut line = String::new();
            reader.read_line(&mut line)
                .map_err(|e| GcpError::NetworkError(format!("Failed to read trailer: {}", e)))?;

            if line.trim().is_empty() {
                break;
            }
        }

        Ok(body)
    }
}

// GitHub file info structure (moved from downloader.rs)
struct GitHubFile {
    name: String,
    path: String,
    file_type: String,
    download_url: Option<String>,
    content: Option<String>,
    encoding: Option<String>,
}

impl GitHubFile {
    fn from_json(json_str: &str) -> Result<Self, GcpError> {
        let mut file = GitHubFile {
            name: String::new(),
            path: String::new(),
            file_type: String::new(),
            download_url: None,
            content: None,
            encoding: None,
        };

        file.name = extract_json_field(json_str, "name");
        file.path = extract_json_field(json_str, "path");
        file.file_type = extract_json_field(json_str, "type");
        file.download_url = extract_optional_json_field(json_str, "download_url");
        file.content = extract_optional_json_field(json_str, "content");
        file.encoding = extract_optional_json_field(json_str, "encoding");

        if file.name.is_empty() {
            return Err(GcpError::ParseError("Invalid JSON format".to_string()));
        }

        Ok(file)
    }
}

// JSON parsing utilities (moved from downloader.rs)
fn extract_json_field(json_str: &str, field: &str) -> String {
    let pattern = format!("\"{}\":", field);
    if let Some(start) = json_str.find(&pattern) {
        let after_field = &json_str[start + pattern.len()..];
        if let Some(quote_start) = after_field.find('"') {
            let after_first_quote = &after_field[quote_start + 1..];
            if let Some(quote_end) = after_first_quote.find('"') {
                return after_first_quote[..quote_end].to_string();
            }
        }
    }
    String::new()
}

fn extract_optional_json_field(json_str: &str, field: &str) -> Option<String> {
    let pattern = format!("\"{}\":", field);
    if let Some(start) = json_str.find(&pattern) {
        let after_field = &json_str[start + pattern.len()..];
        if after_field.trim_start().starts_with("null") {
            return None;
        }
        if let Some(quote_start) = after_field.find('"') {
            let after_first_quote = &after_field[quote_start + 1..];
            if let Some(quote_end) = after_first_quote.find('"') {
                let value = after_first_quote[..quote_end].to_string();
                return if value.is_empty() {
                    None
                } else {
                    Some(value)
                };
            }
        }
    }
    None
}

fn parse_github_file_array(json_str: &str) -> Result<Vec<GitHubFile>, GcpError> {
    let mut files = Vec::new();
    let json_str = json_str.trim();

    if !json_str.starts_with('[') {
        return Err(GcpError::ParseError("Expected JSON array".to_string()));
    }

    let array_content = &json_str[1..json_str.len().saturating_sub(1)];
    let mut current_object = String::new();
    let mut brace_count = 0;
    let mut in_string = false;

    for ch in array_content.chars() {
        if ch == '"' && (current_object.is_empty() || !current_object.ends_with('\\')) {
            in_string = !in_string;
        }

        if !in_string {
            if ch == '{' {
                brace_count += 1;
            } else if ch == '}' {
                brace_count -= 1;
                if brace_count == 0 {
                    current_object.push(ch);
                    if let Ok(file) = GitHubFile::from_json(&current_object) {
                        files.push(file);
                    }
                    current_object.clear();
                    continue;
                }
            }
        }

        current_object.push(ch);
    }

    Ok(files)
}