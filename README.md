# GCP - GitHub Copy Tool (Minimal Version)

[![Rust](https://img.shields.io/badge/rust-1.81+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

GCP (GitHub Copy) - 极简版 GitHub 下载工具，专为轻量级和高效下载而设计。

## ✨ 核心特性

- 📁 **单文件下载**: 从 GitHub 下载单个文件
- 📂 **文件夹下载**: 递归下载整个文件夹
- 🚀 **轻量级**: 优化的二进制大小 (~540KB)
- ⚡ **最小依赖**: 仅依赖 3 个核心库 (attohttpc, base64, wee_alloc)
- 🔧 **内存优化**: 使用 wee_alloc 分配器
- 🎯 **简洁设计**: 专注于核心功能，无冗余特性

## 🚀 快速开始

### 安装

```bash
# 克隆仓库
git clone https://github.com/ishanwen-byte/gcp.git
cd gcp

# 构建发布版本 (极致优化)
cargo build --release

# 进一步压缩 (可选)
upx --best target/release/gcp.exe  # Windows
upx --best target/release/gcp       # Linux/macOS
```

### 基本使用

```bash
# 下载单个文件
./target/release/gcp "https://github.com/rust-lang/rust/blob/main/README.md" ./downloaded_readme.md

# 下载文件夹
./target/release/gcp "https://github.com/rust-lang/rust/tree/main/src" ./rust_source/

# 下载到当前目录 (自动命名)
./target/release/gcp "https://github.com/user/repo/blob/main/config.toml"

# 查看帮助
./target/release/gcp --help
```

## 📖 使用方法

### 命令格式

```bash
gcp <GITHUB_URL> [DESTINATION]
```

### URL 格式支持

#### GitHub.com URL
```bash
# 文件
https://github.com/owner/repo/blob/main/path/to/file.txt

# 文件夹
https://github.com/owner/repo/tree/main/folder-name
```

#### Raw GitHub URL
```bash
# 直接文件下载
https://raw.githubusercontent.com/owner/repo/main/path/to/file.txt
```

### 使用示例

```bash
# 下载 README 文件
gcp "https://github.com/rust-lang/rust/blob/main/README.md"

# 下载整个源码文件夹
gcp "https://github.com/rust-lang/rust/tree/main/src" ./rust-src/

# 下载配置文件到指定位置
gcp "https://github.com/user/repo/blob/main/config.toml" ./my-config.toml
```

## 🏗️ 技术设计

### 核心原则

1. **极简主义**: 移除所有非必需功能，专注核心下载能力
2. **最小依赖**: 仅使用最核心的第三方库
3. **手动解析**: 避免重量级序列化库，手动处理 JSON
4. **内存优化**: 使用 wee_alloc 分配器减少内存占用

### 项目结构

```
gcp/
├── src/
│   ├── main.rs         # CLI 入口点和参数处理
│   ├── lib.rs          # 公共 API 接口
│   ├── github.rs       # GitHub URL 解析和 API 构建
│   ├── downloader.rs   # HTTP 客户端和文件下载
│   └── error.rs        # 最小化错误处理
├── Cargo.toml          # 项目配置 (包含极致优化设置)
├── justfile           # Just 构建命令 (推荐)
├── Makefile           # Make 构建命令 (备选)
├── build.ps1          # PowerShell 构建脚本 (Windows)
└── README.md          # 项目文档
```

### 核心组件

- **src/main.rs**: CLI 参数解析和入口点
- **src/github.rs**: GitHub URL 解析和 API 端点构建
- **src/downloader.rs**: 轻量级 HTTP 客户端和文件操作
- **src/error.rs**: 最小化错误类型定义

### 极致优化配置

项目包含全面的发布优化设置：

- **链接时优化 (LTO)**: 启用
- **Panic 模式**: abort (无展开)
- **代码生成单元**: 1 (最大化优化)
- **符号剥离**: 启用
- **调试信息**: 禁用
- **增量编译**: 禁用

## 🛠️ 构建命令

### 使用 Just (推荐)

```bash
just build          # 构建优化版本
just upx            # 构建并压缩
just size           # 显示二进制大小
just clean          # 清理构建文件
just test           # 运行测试
just all            # 完整构建流程
```

### 使用 Make

```bash
make build          # 构建优化版本
make upx            # 构建并压缩
make size           # 显示二进制大小
make clean          # 清理构建文件
make test           # 运行测试
make all            # 完整构建流程
```

### 使用 PowerShell (Windows)

```powershell
./build.ps1         # 构建优化版本
./build.ps1 -Compress # 构建并压缩
./build.ps1 -All    # 完整构建流程
```

## 🔧 开发环境

### 系统要求

- **Rust**: 1.81.0+ (edition 2024)
- **操作系统**: Windows, Linux, macOS
- **可选工具**: UPX (用于进一步压缩)

### 开发命令

```bash
# 标准构建
cargo build

# 发布构建 (极致优化)
cargo build --release

# 代码检查
cargo check

# 代码格式化
cargo fmt

# 代码质量检查
cargo clippy

# 运行测试
cargo test

# 生成文档
cargo doc --open
```

## 📊 性能指标

### 二进制大小

- **优化后**: ~540KB
- **UPX 压缩后**: ~200KB (可选)
- **内存占用**: 优化 (wee_alloc)

### 依赖分析

```toml
[dependencies]
attohttpc = { version = "0.30.1", default-features = false, features = ["tls"] }
base64 = { version = "0.22", default-features = false, features = ["alloc"] }
wee_alloc = { version = "0.4", default-features = false }
```

**依赖特点:**
- **attohttpc**: 轻量级 HTTP 客户端，最小 TLS 支持
- **base64**: 最小化 base64 编解码，仅 alloc 特性
- **wee_alloc**: 小型内存分配器，减少内存占用

## 🚫 限制说明

### 当前版本限制

- **仅支持公开仓库**: 无认证功能
- **基础功能**: 仅包含核心下载功能
- **无进度显示**: 为减小体积而移除
- **无并发**: 单线程下载
- **错误处理简化**: 最小化错误信息

### 与完整版对比

| 功能 | 极简版 | 完整版 |
|------|--------|--------|
| 二进制大小 | ~540KB | ~5MB |
| 依赖数量 | 3 | 15+ |
| 认证支持 | ❌ | ✅ |
| 并发下载 | ❌ | ✅ |
| 进度显示 | ❌ | ✅ |
| 内存优化 | ✅ | ❌ |

## 🤝 贡献指南

### 开发原则

1. **保持简单**: 拒绝增加不必要的复杂性
2. **体积优先**: 任何新功能都必须考虑对二进制大小的影响
3. **依赖审查**: 新增依赖需要充分的理由
4. **向后兼容**: 保持现有功能的稳定性

### 贡献流程

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/minimal-feature`)
3. 提交更改 (`git commit -m 'Add minimal feature'`)
4. 推送到分支 (`git push origin feature/minimal-feature`)
5. 创建 Pull Request

## 📜 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

- **attohttpc**: 提供轻量级 HTTP 客户端
- **wee_alloc**: 提供小型内存分配器
- **Rust 社区**: 提供优秀的系统编程语言

---

**GCP Minimal** - 极致小巧的 GitHub 下载工具 🚀