//! GitHub client with integrated HTTPS functionality

use crate::base64::Base64Decoder;
use crate::error::{GcpError, GcpResult};
use crate::github::{GitHubUrl, UrlType};
use crate::json::{GitHubFile, parse_github_file_array};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;

/// GitHub client with integrated HTTPS
pub struct GitHubClient {
    tls_connector: native_tls::TlsConnector,
}

impl GitHubClient {
    /// Create new GitHub client
    pub fn new() -> GcpResult<Self> {
        let tls_connector = native_tls::TlsConnector::new().map_err(|e| {
            GcpError::NetworkError(format!("Failed to create TLS connector: {}", e))
        })?;

        Ok(Self { tls_connector })
    }

    /// Download content from GitHub URL
    pub fn download(&self, url: &GitHubUrl, destination: &str) -> GcpResult<()> {
        match url.url_type {
            UrlType::File => self.download_file(url, destination),
            UrlType::Folder => self.download_folder(url, destination),
            UrlType::Repository => Err(GcpError::UnsupportedOperation(
                "Repository downloads not supported in minimal version".to_string(),
            )),
        }
    }

    /// Download a single file
    fn download_file(&self, url: &GitHubUrl, destination: &str) -> GcpResult<()> {
        // Create parent directories if the destination includes a path
        if let Some(parent) = std::path::Path::new(destination).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        // Get file content via GitHub API
        let api_url = url.api_url();
        let (host, path) = self.parse_url(&api_url)?;
        let response = self.http_get(&api_url, &host, &path)?;

        // Parse response and extract content
        let response_text = std::str::from_utf8(&response)
            .map_err(|e| GcpError::ParseError(format!("Invalid UTF-8 in response: {}", e)))?;
        let file_info = GitHubFile::from_json(response_text)?;

        // Write file content
        match file_info.content {
            Some(content) if file_info.encoding.as_deref() == Some("base64") => {
                let clean_content = content
                    .chars()
                    .filter(|c| *c != '\n' && *c != '\r' && *c != '\\')
                    .collect::<String>();
                let decoded = Base64Decoder::decode(&clean_content)
                    .map_err(|e| GcpError::ParseError(format!("Base64 decode error: {}", e)))?;
                std::fs::write(destination, decoded)?;
            }
            Some(content) => {
                // Content is not base64 encoded
                std::fs::write(destination, content)?;
            }
            None if file_info.download_url.is_some() => {
                // Use download URL as fallback
                let download_url = file_info.download_url.unwrap();
                let (host, path) = self.parse_url(&download_url)?;
                let content = self.http_get(&download_url, &host, &path)?;
                std::fs::write(destination, content)?;
            }
            None => {
                return Err(GcpError::NetworkError(
                    "No file content available".to_string(),
                ));
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
        let response = self.http_get(&api_url, &host, &path)?;
        let response_text = std::str::from_utf8(&response)
            .map_err(|e| GcpError::ParseError(format!("Invalid UTF-8 in response: {}", e)))?;
        let files = parse_github_file_array(response_text)?;

        let mut downloaded_count = 0;

        for file in files {
            let file_path = std::path::Path::new(destination).join(&file.name);

            if file.file_type == "file" {
                // Prefer the API contents endpoint per file: it embeds base64
                // content, works when raw.githubusercontent.com is unreachable,
                // and respects the requested ref.
                let api = format!(
                    "{}/repos/{}/{}/contents/{}?ref={}",
                    url.api_base,
                    url.owner,
                    url.repo,
                    file.path,
                    url.ref_.as_deref().unwrap_or("main")
                );
                let (host, path) = self.parse_url(&api)?;
                let response = self.http_get(&api, &host, &path)?;

                match std::str::from_utf8(&response)
                    .map_err(|e| GcpError::ParseError(format!("Invalid UTF-8 in response: {}", e)))
                    .and_then(GitHubFile::from_json)
                {
                    Ok(info) => {
                        if let Some(content) = info.content {
                            if info.encoding.as_deref() == Some("base64") {
                                let clean: String = content
                                    .chars()
                                    .filter(|c| *c != '\n' && *c != '\r' && *c != '\\')
                                    .collect();
                                let decoded = Base64Decoder::decode(&clean).map_err(|e| {
                                    GcpError::ParseError(format!("Base64 decode error: {}", e))
                                })?;
                                std::fs::write(&file_path, decoded)?;
                                downloaded_count += 1;
                            } else {
                                std::fs::write(&file_path, content)?;
                                downloaded_count += 1;
                            }
                        } else if let Some(download_url) = file.download_url {
                            // Fallback to raw URL if API did not embed content
                            let (host, path) = self.parse_url(&download_url)?;
                            let content = self.http_get(&download_url, &host, &path)?;
                            std::fs::write(&file_path, content)?;
                            downloaded_count += 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: skipping {}: {}", file.name, e);
                    }
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
                    api_base: url.api_base.clone(),
                    scheme: url.scheme.clone(),
                    web_host: url.web_host.clone(),
                };

                if let Err(e) =
                    self.download_folder(&sub_url, file_path.to_str().unwrap_or(&file.name))
                {
                    eprintln!("Warning: Failed to download folder {}: {}", file.name, e);
                }
            }
        }

        eprintln!("Downloaded {} files to {}", downloaded_count, destination);
        Ok(())
    }

    /// Parse URL into host and path (http or https)
    fn parse_url(&self, url: &str) -> GcpResult<(String, String)> {
        let remaining = url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
            .ok_or_else(|| GcpError::InvalidUrl("Only HTTP(S) URLs are supported".to_string()))?;

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

    /// Make an HTTP(S) GET request.
    /// `full_url` is used to detect the scheme (http for intranet Gitea);
    /// `host` may carry an explicit port (e.g. "192.168.3.14:3000").
    fn http_get(&self, full_url: &str, host: &str, path: &str) -> GcpResult<Vec<u8>> {
        let is_tls = full_url.starts_with("https://");
        let default_port: u16 = if is_tls { 443 } else { 80 };
        // Split explicit port off the host if present
        let (host_only, port) = match host.rfind(':') {
            Some(i)
                if !host[i + 1..].is_empty()
                    && host[i + 1..].chars().all(|c| c.is_ascii_digit()) =>
            {
                let p = host[i + 1..].parse::<u16>().unwrap_or(default_port);
                (host[..i].to_string(), p)
            }
            _ => (host.to_string(), default_port),
        };

        // Connect to TCP, honoring a proxy from the environment if present.
        // An HTTP proxy is used as a CONNECT tunnel so end-to-end TLS to the
        // origin host is preserved. Proxies are only used for TLS traffic;
        // plain-HTTP intranet requests go direct.
        let tcp_stream = match Self::proxy_from_env().filter(|_| is_tls) {
            Some((proxy_host, proxy_port)) => {
                let stream: TcpStream = TcpStream::connect((proxy_host.as_str(), proxy_port))
                    .map_err(|e| {
                        GcpError::NetworkError(format!(
                            "TCP connection to proxy {}:{} failed: {}",
                            proxy_host, proxy_port, e
                        ))
                    })?;
                // Send CONNECT to establish a tunnel
                let mut stream = stream;
                let connect_req = format!(
                    "CONNECT {host}:{port} HTTP/1.1\r\nHost: {host}:{port}\r\n\r\n",
                    host = host_only,
                    port = port
                );
                stream.write_all(connect_req.as_bytes()).map_err(|e| {
                    GcpError::NetworkError(format!("Failed to send CONNECT: {}", e))
                })?;
                let mut reader = BufReader::new(&mut stream);
                let mut status = String::new();
                reader.read_line(&mut status).map_err(|e| {
                    GcpError::NetworkError(format!("Failed to read CONNECT response: {}", e))
                })?;
                if !status.starts_with("HTTP/1.1 2") && !status.starts_with("HTTP/1.0 2") {
                    return Err(GcpError::NetworkError(format!(
                        "Proxy CONNECT failed: {}",
                        status.trim()
                    )));
                }
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).map_err(|e| {
                        GcpError::NetworkError(format!("Failed to read proxy header: {}", e))
                    })?;
                    if line.trim().is_empty() {
                        break;
                    }
                }
                stream
            }
            None => TcpStream::connect((host_only.as_str(), port))
                .map_err(|e| GcpError::NetworkError(format!("TCP connection failed: {}", e)))?,
        };

        // Percent-encode non-ASCII bytes in the request line: raw UTF-8 in a
        // request target is a protocol violation and gets a 400 from GitHub.
        let encoded_path = Self::percent_encode_path(path);
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: gcp/0.1.0\r\nConnection: close\r\nAccept: */*\r\n\r\n",
            encoded_path, host
        );

        if is_tls {
            let mut tls_stream = self
                .tls_connector
                .connect(&host_only, tcp_stream)
                .map_err(|e| GcpError::NetworkError(format!("TLS handshake failed: {}", e)))?;
            tls_stream
                .write_all(request.as_bytes())
                .map_err(|e| GcpError::NetworkError(format!("Failed to send request: {}", e)))?;
            self.read_http_response(&mut tls_stream)
        } else {
            let mut stream = tcp_stream;
            stream
                .write_all(request.as_bytes())
                .map_err(|e| GcpError::NetworkError(format!("Failed to send request: {}", e)))?;
            self.read_http_response(&mut stream)
        }
    }

    /// Read `HTTPS_PROXY`/`https_proxy` (or `ALL_PROXY`) from the environment
    /// and parse it into (host, port). Only http:// proxies are supported
    /// (used as CONNECT tunnels).
    fn proxy_from_env() -> Option<(String, u16)> {
        let raw = std::env::var("HTTPS_PROXY")
            .or_else(|_| std::env::var("https_proxy"))
            .or_else(|_| std::env::var("ALL_PROXY"))
            .or_else(|_| std::env::var("all_proxy"))
            .ok()?
            .trim()
            .to_string();

        if raw.is_empty() {
            return None;
        }

        // Strip scheme
        let rest = raw
            .strip_prefix("http://")
            .or_else(|| raw.strip_prefix("https://"))
            .unwrap_or(&raw);

        // Strip trailing path and userinfo
        let rest = rest.split('/').next().unwrap_or(rest);
        let rest = match rest.rfind('@') {
            Some(i) => &rest[i + 1..],
            None => rest,
        };

        let (host, port) = match rest.rfind(':') {
            Some(i) => {
                let p = rest[i + 1..].parse::<u16>().ok()?;
                (rest[..i].to_string(), p)
            }
            None => (rest.to_string(), 80),
        };

        if host.is_empty() {
            None
        } else {
            Some((host, port))
        }
    }

    /// Percent-encode the request target so only valid ASCII reaches the wire.
    /// Already-encoded sequences are left alone ('%' is preserved as-is),
    /// unreserved characters pass through, everything else is %XX-encoded.
    fn percent_encode_path(path: &str) -> String {
        let mut out = String::with_capacity(path.len());
        for byte in path.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' => out.push(byte as char),
                b'/' | b'?' | b'&' | b'=' | b'%' | b'-' | b'_' | b'.' | b'~' | b'+' | b':'
                | b'@' => out.push(byte as char),
                _ => out.push_str(&format!("%{:02X}", byte)),
            }
        }
        out
    }

    /// Read and parse HTTP response
    fn read_http_response(&self, stream: &mut impl Read) -> GcpResult<Vec<u8>> {
        let mut reader = BufReader::new(stream);

        // Read status line
        let mut status_line = String::new();
        reader
            .read_line(&mut status_line)
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
            reader
                .read_line(&mut line)
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

            if line.to_lowercase().starts_with("transfer-encoding:")
                && line.to_lowercase().contains("chunked")
            {
                chunked = true;
            }
        }

        // Read body based on transfer encoding
        if let Some(length) = content_length {
            let mut body = vec![0u8; length];
            reader
                .read_exact(&mut body)
                .map_err(|e| GcpError::NetworkError(format!("Failed to read body: {}", e)))?;
            Ok(body)
        } else if chunked {
            self.read_chunked_body(&mut reader)
        } else {
            let mut body = Vec::new();
            reader
                .read_to_end(&mut body)
                .map_err(|e| GcpError::NetworkError(format!("Failed to read body: {}", e)))?;
            Ok(body)
        }
    }

    /// Read chunked transfer encoding body
    fn read_chunked_body(&self, reader: &mut impl BufRead) -> GcpResult<Vec<u8>> {
        let mut body = Vec::new();

        loop {
            let mut chunk_size_line = String::new();
            reader
                .read_line(&mut chunk_size_line)
                .map_err(|e| GcpError::NetworkError(format!("Failed to read chunk size: {}", e)))?;

            let chunk_size = usize::from_str_radix(chunk_size_line.trim(), 16)
                .map_err(|_| GcpError::ParseError("Invalid chunk size".to_string()))?;

            if chunk_size == 0 {
                break;
            }

            let mut chunk = vec![0u8; chunk_size];
            reader
                .read_exact(&mut chunk)
                .map_err(|e| GcpError::NetworkError(format!("Failed to read chunk: {}", e)))?;
            body.extend_from_slice(&chunk);

            let mut crlf = [0u8; 2];
            reader
                .read_exact(&mut crlf)
                .map_err(|e| GcpError::NetworkError(format!("Failed to read CRLF: {}", e)))?;
        }

        // Skip trailer headers
        loop {
            let mut line = String::new();
            reader
                .read_line(&mut line)
                .map_err(|e| GcpError::NetworkError(format!("Failed to read trailer: {}", e)))?;

            if line.trim().is_empty() {
                break;
            }
        }

        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url() {
        let client = GitHubClient::new().unwrap();
        let (host, path) = client
            .parse_url("https://api.github.com/repos/a/b/contents/c?ref=main")
            .unwrap();
        assert_eq!(host, "api.github.com");
        assert_eq!(path, "/repos/a/b/contents/c?ref=main");
    }

    #[test]
    fn test_parse_url_no_path() {
        let client = GitHubClient::new().unwrap();
        assert!(client.parse_url("https://github.com").is_err());
    }

    #[test]
    fn test_parse_url_accepts_http_for_gitea() {
        // Plain http is valid for intranet Gitea instances
        let client = GitHubClient::new().unwrap();
        let (host, path) = client
            .parse_url("http://192.168.3.14:3000/api/v1/repos/a/b/contents/c?ref=main")
            .unwrap();
        assert_eq!(host, "192.168.3.14:3000");
        assert_eq!(path, "/api/v1/repos/a/b/contents/c?ref=main");
    }

    #[test]
    fn test_parse_url_rejects_non_http_scheme() {
        let client = GitHubClient::new().unwrap();
        assert!(client.parse_url("ftp://api.github.com/x").is_err());
        assert!(client.parse_url("api.github.com/x").is_err());
    }

    #[test]
    fn test_read_http_response_content_length() {
        let client = GitHubClient::new().unwrap();
        let raw = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello";
        let body = client.read_http_response(&mut &raw[..]).unwrap();
        assert_eq!(body, b"hello");
    }

    #[test]
    fn test_read_http_response_chunked() {
        let client = GitHubClient::new().unwrap();
        let raw = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\r\n6\r\n world\r\n0\r\n\r\n";
        let body = client.read_http_response(&mut &raw[..]).unwrap();
        assert_eq!(body, b"hello world");
    }

    #[test]
    fn test_read_http_response_error_status() {
        let client = GitHubClient::new().unwrap();
        let raw = b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
        assert!(client.read_http_response(&mut &raw[..]).is_err());
    }

    #[test]
    fn test_download_file_creates_parent_dirs() {
        // Validates that a destination with a missing parent directory
        // is created rather than failing with "path not found".
        // Uses an invalid host so the network call fails fast, after the
        // parent-dir creation has already happened.
        let client = GitHubClient::new().unwrap();
        let dir = std::env::temp_dir().join("gcp_test_missing_parent");
        let _ = std::fs::remove_dir_all(&dir);

        let url = GitHubUrl {
            owner: "octocat".to_string(),
            repo: "Hello-World".to_string(),
            path: Some("README".to_string()),
            ref_: Some("master".to_string()),
            url_type: UrlType::File,
            raw_url: String::new(),
            api_base: "https://api.github.com".to_string(),
            scheme: "https".to_string(),
            web_host: "github.com".to_string(),
        };

        let dest = dir.join("nested").join("deep").join("out.txt");
        let dest_str = dest.to_str().unwrap().to_string();

        // The download itself fails (invalid host), but parent dirs must exist.
        let _ = client.download_file(&url, &dest_str);
        assert!(dir.join("nested").join("deep").is_dir());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_percent_encode_path_ascii_passthrough() {
        // Plain ASCII path with query stays untouched
        let p = "/repos/a/b/contents/src/lib.rs?ref=main";
        assert_eq!(GitHubClient::percent_encode_path(p), p);
    }

    #[test]
    fn test_percent_encode_path_cjk() {
        // CJK characters must be %XX-encoded as their UTF-8 bytes
        let out = GitHubClient::percent_encode_path("/repos/a/b/contents/中文.md?ref=main");
        assert!(
            out.starts_with("/repos/a/b/contents/%E4%B8%AD%E6%96%87.md?ref=main"),
            "got: {out}"
        );
        assert!(out.is_ascii());
    }

    #[test]
    fn test_percent_encode_path_preserves_existing_encoding() {
        // Already-percent-encoded input must not be double-encoded
        let p = "/repos/a/b/contents/%E4%B8%AD.md?ref=main";
        assert_eq!(GitHubClient::percent_encode_path(p), p);
    }

    #[test]
    fn test_percent_encode_path_space() {
        assert_eq!(GitHubClient::percent_encode_path("/a b.txt"), "/a%20b.txt");
    }
}
