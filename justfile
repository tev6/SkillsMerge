# SkillsMerge 开发命令
# 安装 just: cargo install just

default:
    @just --list

# 检查代码编译
check:
    cargo check --workspace --all-features

# 运行所有测试
test:
    cargo test --workspace --all-features

# 运行测试并显示输出
test-verbose:
    cargo test --workspace --all-features -- --nocapture

# 运行 clippy 检查
lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# 格式化代码
fmt:
    cargo fmt --all

# 检查格式（CI 模式）
fmt-check:
    cargo fmt --all -- --check

# 安全审计
audit:
    cargo audit

# 完整 CI 检查（本机运行）
ci: lint fmt-check test

# Release 构建
release:
    cargo build --release

# 运行示例
example:
    cargo run --example basic_merge

# 清理构建产物
clean:
    cargo clean

# 运行单个测试
test-unit:
    cargo test --test unit_test

# 运行集成测试
test-integration:
    cargo test --test integration_test

# 查看文档
doc:
    cargo doc --open

# 安装到系统
install:
    cargo install --path .
