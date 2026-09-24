 CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

GCP (GitHub Copy) is a minimal command-line tool for downloading files and folders from public GitHub **and Gitea** repositories. It uses hand-written HTTP/1.1 over native-tls (no HTTP client library), hand-written JSON field extraction and base64 decoding, keeping a single runtime dependency and a ~213 KB Windows binary.

## Common Development Commands

```bash
cargo build --release   # optimized build
cargo test              # 31 unit tests (inline in src)
cargo fmt               # formatting (enforced, .rustfmt.toml: edition 2024, LF)
cargo clippy --all-targets  # linting (must be warning-free)
```

Build helpers: `just build` / `just test` (justfile), `make build` (Makefile), `./build.ps1` (Windows). UPX compression: `just build-upx` (requires `upx` on PATH).

## Architecture

| Module | Responsibility |
|---|---|
| `src/main.rs` | CLI entry: arg parsing, usage text, exit codes |
| `src/lib.rs` | Public API `download_from_github(url, dest)`; destination normalization (default filename, trailing-separator stripping) |
| `src/github.rs` | URL parsing for github.com / raw.githubusercontent.com / Gitea hosts; derives `api_base` (GitHub: `api.github.com`, Gitea: `<host>/api/v1`); `api_url()` with `?ref=` |
| `src/client.rs` | `http_get`: scheme/port detection, proxy CONNECT tunnel (env `HTTPS_PROXY`/`ALL_PROXY`, TLS-only), percent-encoding of non-ASCII request targets, HTTP/1.1 response parsing (content-length, chunked); file/folder download with base64 decode |
| `src/json.rs` | JSON field extraction with full string unescaping (`\uXXXX` incl. surrogate pairs); **byte-offset slicing** (`char_indices`) — multi-byte UTF-8 in values must never panic |
| `src/base64.rs` | Hand-written base64 decoder |
| `src/error.rs` | `GcpError` enum, io::Error conversion |

## Key Behaviors (verified end-to-end)

- Folder downloads fetch per-file content via the contents API (embeds base64), falling back to `download_url` for files >1MB (API returns `content: null`)
- Destination parent directories are created automatically
- Plain `http://` is accepted for intranet Gitea; port parsed from host (`192.168.3.14:3000`)
- Non-ASCII paths (CJK filenames) are percent-encoded on the wire (RFC 3986); raw UTF-8 in a request line gets 400 from GitHub
- Proxy env vars are honored for TLS traffic only; empty-string values are ignored

## Testing Tips

- Intranet Gitea (`http://192.168.3.14:3000`) serves as a rate-limit-free GitHub API-compatible test bed (repo `goliath/Dtodo` has CJK filenames and a 26MB APK useful for hash verification)
- GitHub unauthenticated limit is 60 req/h/IP; when exhausted use the Gitea instance
- CLI exit codes: 0 success/help, 1 usage/network/parse errors

## Conventions

- Comments and commit messages in English; README in Chinese
- Keep dependencies minimal: new deps need strong justification
- `panic = "abort"` in release: panics are bugs (see the UTF-8 slicing incident), not control flow
