# Build stage
FROM rust:1.84-bookworm AS builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for caching
COPY Cargo.toml Cargo.lock ./

# Create dummy src for dependency caching
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true
RUN rm -rf src

# Copy actual source and build
COPY src ./src
COPY benches ./benches
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/solana-mcp-server /app/solana-mcp-server
COPY config.json /app/config.json

RUN useradd -m -u 65532 appuser && chown -R appuser:appuser /app
USER appuser

EXPOSE 3000 8080

ENV RUST_LOG=info
ENTRYPOINT ["/app/solana-mcp-server"]
CMD ["web", "--port", "3000"]
