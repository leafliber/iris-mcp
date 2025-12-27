# 跨平台编译指南

本项目支持为 macOS、Linux 和 Windows 生成 release 版本。

## 📦 方案一：使用 Makefile（推荐）

### 安装依赖
```bash
# 安装 zig（用于跨平台编译）
brew install zig

# 安装 cargo-zigbuild
cargo install cargo-zigbuild
```

### 构建命令
```bash
# 仅构建当前平台（macOS）
make release-macos

# 构建 Linux 版本
make release-linux

# 构建 Windows 版本
make release-windows

# 构建所有平台
make release-all

# 查看帮助
make help
```

## 🔧 方案二：使用脚本

### 使用 cargo-zigbuild（推荐）
```bash
# 安装依赖
brew install zig
cargo install cargo-zigbuild

# 运行构建脚本
./build-with-zig.sh
```

### 使用原生交叉编译
```bash
# 运行构建脚本
./build-release.sh
```

## 📋 支持的平台

- ✅ **macOS ARM64** (Apple Silicon)
- ✅ **Linux x86_64** (Intel/AMD 64位)
- ✅ **Linux ARM64** (ARM 64位)
- ✅ **Windows x86_64** (Intel/AMD 64位)

## 📂 输出文件

构建完成后，所有二进制文件将位于 `release-builds/` 目录：

```
release-builds/
├── iris-mcp-macos-arm64               # macOS ARM64
├── iris-mcp-linux-x86_64              # Linux x86_64
├── iris-mcp-linux-aarch64             # Linux ARM64
└── iris-mcp-windows-x86_64.exe        # Windows x86_64
```

## ⚠️ 注意事项

### 1. 平台特定依赖
- macOS 版本依赖 `core-graphics` 和 `core-foundation`
- 这些依赖仅在 macOS 上可用，交叉编译到其他平台时会使用对应的平台库

### 2. 测试建议
- 交叉编译的二进制文件应在目标平台上测试
- macOS 上编译的 Linux/Windows 版本可能需要在真实环境中验证

### 3. 手动构建

如果自动化工具失败，可以手动构建：

```bash
# 添加目标平台
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-pc-windows-gnu

# 构建特定平台
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

## 🐳 使用 Docker（终极方案）

如果遇到交叉编译问题，可以使用 Docker：

```dockerfile
# 创建 Dockerfile
FROM rust:latest
WORKDIR /app
COPY . .
RUN cargo build --release
```

然后在不同平台的机器上构建：
```bash
docker build -t iris-mcp-builder .
docker run --rm -v $(pwd)/target:/app/target iris-mcp-builder
```

## 🚀 GitHub Actions 自动构建

推荐在 GitHub Actions 中配置自动构建多平台版本，详见 `.github/workflows/release.yml`（如果需要可以创建）。
