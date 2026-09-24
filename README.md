# GCP - GitHub Copy Tool (Minimal)

[![Rust](https://img.shields.io/badge/rust-2024--edition-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

从公开 GitHub / Gitea 仓库下载文件与文件夹的极简命令行工具。

## 特性

- **单文件下载** - 支持 `blob` 与 `raw.githubusercontent.com` URL
- **文件夹递归下载** - 支持 `tree` URL，自动还原子目录结构
- **分支/标签感知** - URL 中指定的 ref 会正确传递给 GitHub API（`?ref=...`）
- **二进制安全** - 文件按字节写入，图片等二进制文件不损坏
- **非 ASCII 路径支持** - 中文等非 ASCII 文件名/路径自动百分号编码（RFC 3986）
- **受限网络友好** - 文件夹下载统一走 `api.github.com` 内容端点（内嵌 base64），不依赖 `raw.githubusercontent.com` 的可达性
- **代理支持** - 自动读取 `HTTPS_PROXY` / `https_proxy` / `ALL_PROXY` 环境变量（HTTP 代理，CONNECT 隧道，端到端 TLS 不受影响）
- **Gitea 兼容** - 支持任意 Gitea 实例（含内网 http:// 与自定义端口），URL 格式 `http(s)://<host>/<owner>/<repo>/src/<ref>/<path>`
- **极小体积** - 发布版约 213 KB（Windows x64），仅一个运行时依赖

## 安装

```bash
git clone https://github.com/ishanwen-byte/gcp.git
cd gcp
cargo build --release
```

可选压缩：

```bash
upx --best target/release/gcp.exe
```

## 使用

```
gcp <github_url> [destination]
```

省略 `destination` 时使用原始文件/文件夹名。

```bash
# 下载单个文件
gcp https://github.com/octocat/Hello-World/blob/master/README

# 下载到指定路径
gcp https://github.com/octocat/Hello-World/blob/master/README hello.txt

# 递归下载文件夹（含子目录）
gcp https://github.com/octocat/Hello-World/tree/master/src ./src

# 指定分支/标签
gcp https://github.com/user/repo/tree/v1.0.0/docs ./docs

# raw URL 也支持
gcp https://raw.githubusercontent.com/user/repo/main/file.txt

# Gitea 实例（内网/自建）
gcp http://192.168.3.14:3000/goliath/repo/src/main/docs/readme.md
gcp http://gitea.example.com:3000/owner/repo/src/v1.0.0/release ./release
```

## 工作原理

1. 解析 GitHub/Gitea URL（owner / repo / ref / path，Gitea 推导 `api_base`）
2. 请求 `<api_base>/repos/{owner}/{repo}/contents/{path}?ref={ref}`（GitHub 为 api.github.com，Gitea 为 `<host>/api/v1`）
3. 手写 JSON 提取器解析响应（完整字符串反转义，支持 `\uXXXX` 代理对）
4. base64 解码内容并按字节写入磁盘

文件夹下载对每个文件重复步骤 2-4，子目录递归处理；若 API 未内嵌内容则回退到 `download_url`。

## 项目结构

```
src/
├── main.rs     # CLI 入口
├── lib.rs      # 公共 API（download_from_github）
├── github.rs   # URL 解析与 API 端点构建
├── client.rs   # HTTPS 客户端（native-tls + 手写 HTTP/1.1）
├── json.rs     # 手写 JSON 字段提取与反转义
├── base64.rs   # 手写 base64 解码
└── error.rs    # 错误类型
```

## 依赖

| 依赖 | 用途 |
|---|---|
| `native-tls` | TLS（Windows 用 schannel，macOS 用 Security.framework，Linux 用 OpenSSL；Gitea http:// 场景不经过 TLS） |

base64 与 JSON 解析均为手写实现，无第三方依赖。

## 限制

- 仅支持公开仓库（无认证）
- 单线程下载
- GitHub API 未认证限额为 60 请求/小时/IP，大文件夹下载可能中途触发限流（可配置代理或等待重置）
- 整仓库下载不支持（clone 请用 git）

## 开发

```bash
cargo build --release   # 构建优化版
cargo test              # 运行测试（23 个）
cargo fmt               # 格式化
cargo clippy            # 静态检查
```

构建脚本：`justfile` / `Makefile` / `build.ps1`。

## 许可证

[MIT](LICENSE)
