# === 构建阶段 (Builder) ===
# 使用 Rust 官方镜像，基于 Debian Bookworm
FROM rust:1.75-bookworm as builder

# 安装必要的系统依赖 (Solana SDK 和网络库需要)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libudev-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# 为了利用 Docker 缓存机制加速构建，我们先只复制 Cargo 配置文件编译依赖
# 1. 创建空的 scavenger 目录结构
RUN mkdir scavenger
WORKDIR /usr/src/app/scavenger

# 2. 复制依赖清单
COPY scavenger/Cargo.toml scavenger/Cargo.lock ./

# 3. 创建一个假的 main.rs 来预编译依赖
RUN mkdir src && echo "fn main() {}" > src/main.rs

# 4. 编译依赖 (这一步会被缓存，除非 Cargo.toml 变动)
# 注意：我们需要确保 target 目录存在，避免 clean 错误
RUN cargo build --release --bin scavenger

# 5. 清理假的构建产物
RUN rm -f target/release/deps/scavenger*
RUN rm -rf src

# === 源码编译 ===
# 6. 复制真正的源代码
COPY scavenger/src ./src

# 7. 编译正式项目
RUN cargo build --release --bin scavenger

# === 运行阶段 (Runner) ===
# 使用轻量级 Debian Slim 镜像
FROM python:3.9-slim-bookworm

WORKDIR /app

# 安装系统运行时依赖
RUN apt-get update && apt-get install -y \
    libssl3 \
    libudev1 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 从构建阶段复制编译好的二进制文件
COPY --from=builder /usr/src/app/scavenger/target/release/scavenger /usr/local/bin/scavenger

# 复制 Commander 和 Configs
COPY commander /app/commander

# 创建日志目录
RUN mkdir -p /app/logs

# 设置环境变量
ENV RUST_LOG=info

# 启动命令 (默认使用 Python Commander 启动 arb 策略)
CMD ["python3", "commander/main.py", "--strategy", "arb"]
