# GCP - GitHub Copy Tool

[![Rust](https://img.shields.io/badge/rust-1.74+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/gcp)](https://crates.io/crates/gcp)
[![Build Status](https://img.shields.io/github/workflow/CI/badge.svg)](https://github.com/ishanwen-byte/gcp/actions/workflows/ci.yml)

GCP (GitHub Copy) 是一个专业的命令行工具，用于从 GitHub 仓库下载文件和文件夹，类似于 `cp` 命令但专门针对 GitHub 内容进行优化。

## ✨ 核心功能

- 📁 **单文件下载**: 从 GitHub 下载单个文件
- 📂 **文件夹下载**: 递归下载整个文件夹
- 🔐 **GitHub 认证**: 支持私人仓库访问
- ⚡ **并发下载**: 多文件同时下载
- 🎯 **模式过滤**: 支持通配符模式匹配
- 📊 **进度显示**: 实时下载进度
- 🔄 **智能重试**: 自动错误恢复
- 📝 **干运行**: 预览下载操作

## 🚀 快速开始

### 安装

```bash
# 克隆仓库
git clone https://github.com/ishanwen-byte/gcp.git
cd gcp

# 构建发布版本
cargo build --release

# 安装到系统路径
cargo install --path .
```

### 基本使用

```bash
# 下载单个文件
gcp "https://github.com/rust-lang/rust/blob/main/README.md" ./downloaded_readme.md

# 下载文件夹
gcp "https://github.com/rust-lang/rust/tree/main/src" ./rust_source/

# 使用认证下载私有内容
GITHUB_TOKEN=your_token gcp "private-repo-file-URL" ./downloads/

# 下载到当前目录
gcp "https://github.com/user/repo/blob/main/config.toml"

# 显示进度条和详细信息
gcp --progress --verbose "folder-URL" ./downloads/
```

## 📖 详细用法

### 命令行选项

```bash
gcp [OPTIONS] <SOURCE> [DESTINATION]
```

#### 必需参数
- `<SOURCE>`: GitHub URL（文件或文件夹）
- `[DESTINATION]`: 本地目标路径（默认：当前目录）

#### 主要选项
- `-t, --auth-token <TOKEN>`: GitHub 认证令牌
- `-v, --verbose`: 详细输出模式
- `-q, --quiet`: 静默模式
- `--dry-run`: 预览操作，不实际下载
- `--progress`: 显示进度条
- `-f, --force`: 强制覆盖现有文件

#### 高级选项
- `--preserve-timestamps`: 保留文件修改时间
- `--include <PATTERN>`: 只下载匹配的文件（支持 glob 模式）
- `--exclude <PATTERN>`: 排除匹配的文件
- `--max-concurrent <N>`: 最大并发下载数（默认：10）
- `--timeout <SECONDS>`: 请求超时时间（默认：30）
- `--retry <N>`: 最大重试次数（默认：3）
- `--cache-dir <DIR>`: 缓存目录
- `--no-cache`: 禁用缓存

### URL 格式支持

#### GitHub.com 格式
```bash
# 文件 URL
https://github.com/owner/repo/blob/main/path/to/file.txt

# 文件夹 URL
https://github.com/owner/repo/tree/main/folder-name
```

#### Raw GitHub 格式
```bash
# 直接文件下载
https://raw.githubusercontent.com/owner/repo/main/path/to/file.txt
```

### 高级使用示例

```bash
# 下载特定类型文件
gcp --include "*.rs" --include "*.toml" "repo-url" ./src/

# 排除测试文件
gcp --exclude "*.test.rs" --exclude "*_test.*" "repo-url" ./code/

# 高性能下载
gcp --max-concurrent 20 --timeout 60 --retry 5 "large-repo-url" ./downloads/

# 使用认证和自定义设置
GITHUB_TOKEN=ghp_xxx gcp \
  --progress \
  --include "*.md" \
  --cache-dir ~/.gcp-cache \
  "private-repo-url" ./docs/

# 批量下载模式
gcp --progress --verbose --dry-run \
  "https://github.com/user/repo/tree/main/configs" \
  ./configs/
```

## 🔧 认证设置

### 环境变量认证
```bash
export GITHUB_TOKEN="your_github_personal_access_token"
gcp "private-repo-file-URL" ./downloaded_file
```

### 命令行参数认证
```bash
gcp -t "your_github_personal_access_token" "repo-file-URL" ./downloaded_file
```

### Personal Access Token 创建
1. 访问 [GitHub Settings > Developer settings > Personal access tokens](https://github.com/settings/tokens)
2. 点击 "Generate new token"
3. 选择适当的权限范围（`repo` 权限通常足够）
4. 复制生成的 token

## 🏗️ 技术架构

### 项目结构
```
gcp/
├── src/
│   ├── main.rs              # CLI 入口点
│   ├── lib.rs               # 库根模块
│   ├── error.rs             # 错误处理
│   ├── github/              # GitHub 集成
│   │   ├── mod.rs           # 模块导出
│   │   ├── auth.rs          # 认证处理
│   │   ├── client.rs        # GitHub API 客户端
│   │   └── types.rs         # 类型定义
│   ├── downloader/          # 下载器模块
│   │   ├── mod.rs           # 模块导出
│   │   ├── file.rs          # 文件下载器
│   │   ├── folder.rs        # 文件夹下载器
│   │   └── progress.rs      # 进度报告
│   └── filesystem/           # 文件系统工具
│       └── utils.rs         # 文件系统实用工具
├── .cargo/                   # Cargo 配置
│   ├── config.toml           # 主配置
│   ├── config.dev.toml       # 开发配置
│   ├── config.release.toml   # 发布配置
│   └── config.bench.toml     # 基准测试配置
├── scripts/                  # 脚本工具
│   ├── test_configs.sh       # 配置测试脚本
│   └── test_configs.ps1      # PowerShell 测试脚本
├── tests/                    # 测试文件
├── Cargo.toml                # 项目配置
└── README.md                  # 项目文档
```

### 核心组件

- **GitHubClient**: GitHub API 交互
- **FileDownloader**: 单文件下载逻辑
- **FolderDownloader**: 文件夹递归下载
- **ProgressReporter**: 下载进度可视化
- **ErrorHandling**: 完善的错误处理和重试机制

### 依赖项

- **clap**: 命令行参数解析
- **tokio**: 异步运行时
- **reqwest**: HTTP 客户端
- **octocrab**: GitHub API 客户端
- **tracing**: 结构化日志
- **indicatif**: 进度条显示
- **serde**: 序列化支持
- **chrono**: 时间处理

## 🛠️ 开发

### 构建要求

- Rust 1.74.0+
- Cargo 1.74.0+

### 开发设置

```bash
# 克隆仓库
git clone https://github.com/ishanwen-byte/gcp.git
cd gcp

# 安装开发依赖
cargo build

# 运行测试
cargo test

# 代码格式化
cargo fmt

# 静态分析
cargo clippy
```

### 项目配置

项目包含多个 Cargo 配置文件：

- **`config.toml`**: 默认配置
- **`config.dev.toml`**: 开发优化配置
- **config.release.toml`**: 发布优化配置
- **config.bench.toml`**: 基准测试配置

## 📝 测试

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_file_download
cargo test test_folder_download

# 运行基准测试
cargo bench

# 配置测试脚本
./scripts/test_configs.sh
# Windows 用户
./scripts/test_configs.ps1
```

### 测试覆盖

- ✅ 单文件下载功能
- ✅ 文件夹下载功能
- ✅ URL 解析和验证
- ✅ 认证机制
- ✅ 错误处理
- ✅ 配置选项
- ✅ 并发下载

## 🐛 故障排除

### 常见问题

#### 1. 认证失败
```bash
# 检查 token 格式
echo $GITHUB_TOKEN | cut -c1-10

# 验证 token 权限
curl -H "Authorization: token $GITHUB_TOKEN" \
     https://api.github.com/user
```

#### 2. 网络连接问题
```bash
# 检查 GitHub API 可访问性
curl -I https://api.github.com/rate_limit

# 测试特定仓库访问
curl -I https://api.github.com/repos/rust-lang/rust
```

#### 3. 文件权限问题
```bash
# 检查目标目录权限
ls -la ./downloads/

# 修复权限（如果需要）
chmod 755 ./downloads/
```

#### 4. 大文件下载
```bash
# 增加超时时间
gcp --timeout 120 "large-file-url" ./

# 调整并发数
gcp --max-concurrent 5 "large-repo-url" ./
```

## 📄 更新日志

### v0.1.0 (2024-10-15)
- 🎉 首次发布
- ✅ 单文件下载功能
- ✅ 文件夹下载核心实现
- ✅ GitHub 认证支持
- ✅ 进度显示和错误处理
- ✅ 模式过滤和并发下载
- ✅ 完整的 CLI 接口
- ✅ 详细的配置选项

## 🤝 贡献

欢迎贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详细信息。

### 开发流程

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 📜 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

- 感谢所有贡献者的努力
- 感谢 [octocrab](https://github.com/XAMPPRocky/octocrab) 提供优秀的 GitHub API 客户端
- 感谢 Rust 社区的优秀工具和库

---

**GCP** - 让 GitHub 内容下载变得简单高效！ 🚀