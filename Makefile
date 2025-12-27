.PHONY: all clean release-all release-linux release-windows release-macos help

# 默认目标
all: release-macos

# 帮助信息
help:
	@echo "可用的构建命令："
	@echo "  make release-macos    - 构建 macOS 版本"
	@echo "  make release-linux    - 构建 Linux 版本"
	@echo "  make release-windows  - 构建 Windows 版本"
	@echo "  make release-all      - 构建所有平台版本"
	@echo "  make clean            - 清理构建文件"
	@echo ""
	@echo "推荐使用 cargo-zigbuild 进行跨平台编译："
	@echo "  brew install zig"
	@echo "  cargo install cargo-zigbuild"
	@echo "  ./build-with-zig.sh"

# 创建输出目录
release-builds:
	mkdir -p release-builds

# 构建 macOS 版本
release-macos: release-builds
	@echo "📦 构建 macOS 版本..."
	cargo build --release
	cp target/release/iris-mcp release-builds/iris-mcp-macos-$$(uname -m)
	@echo "✓ macOS 版本已生成: release-builds/iris-mcp-macos-$$(uname -m)"

# 构建 Linux x86_64 版本
release-linux: release-builds
	@echo "📦 构建 Linux x86_64 版本..."
	@if command -v cargo-zigbuild >/dev/null 2>&1; then \
		cargo zigbuild --release --target x86_64-unknown-linux-gnu && \
		cp target/x86_64-unknown-linux-gnu/release/iris-mcp release-builds/iris-mcp-linux-x86_64 && \
		echo "✓ Linux x86_64 版本已生成"; \
	else \
		echo "❌ 需要安装 cargo-zigbuild: cargo install cargo-zigbuild"; \
		exit 1; \
	fi

# 构建 Linux ARM64 版本
release-linux-arm64: release-builds
	@echo "📦 构建 Linux ARM64 版本..."
	@if command -v cargo-zigbuild >/dev/null 2>&1; then \
		cargo zigbuild --release --target aarch64-unknown-linux-gnu && \
		cp target/aarch64-unknown-linux-gnu/release/iris-mcp release-builds/iris-mcp-linux-aarch64 && \
		echo "✓ Linux ARM64 版本已生成"; \
	else \
		echo "❌ 需要安装 cargo-zigbuild"; \
		exit 1; \
	fi

# 构建 Windows x86_64 版本
release-windows: release-builds
	@echo "📦 构建 Windows x86_64 版本..."
	@if command -v cargo-zigbuild >/dev/null 2>&1; then \
		cargo zigbuild --release --target x86_64-pc-windows-gnu && \
		cp target/x86_64-pc-windows-gnu/release/iris-mcp.exe release-builds/iris-mcp-windows-x86_64.exe && \
		echo "✓ Windows x86_64 版本已生成"; \
	else \
		echo "❌ 需要安装 cargo-zigbuild"; \
		exit 1; \
	fi

# 构建所有平台
release-all: release-macos release-linux release-linux-arm64 release-windows
	@echo ""
	@echo "✨ 所有平台构建完成！"
	@echo "生成的文件："
	@ls -lh release-builds/ | tail -n +2 | awk '{print "  " $$9 " (" $$5 ")"}'

# 清理构建文件
clean:
	cargo clean
	rm -rf release-builds
	@echo "✓ 清理完成"
