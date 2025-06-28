# Multi-stage build for optimized image size
FROM rust:slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
COPY services/ ./services/

# Set build-time environment variables to avoid missing env vars during compilation
ENV SQLX_OFFLINE=true
# Set a dummy DATABASE_URL to satisfy sqlx compilation checks
ENV DATABASE_URL="postgresql://postgres:postgres@localhost/dummy"

# Build dependencies first for better caching
RUN cargo build --release --bin workflow-engine || \
    (echo "Build failed. Checking for common issues..." && \
     cargo check --bin workflow-engine 2>&1 | head -50)

# Build the GraphQL gateway
RUN cargo build --release --bin graphql-gateway || \
    (echo "Gateway build failed. Checking for common issues..." && \
     cargo check --bin graphql-gateway 2>&1 | head -50)

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    libpq5 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 aiworkflow

# Copy the binary from builder
COPY --from=builder /app/target/release/workflow-engine /usr/local/bin/ai-workflow

# Create necessary directories
RUN mkdir -p /app/logs /app/workflows && \
    chown -R aiworkflow:aiworkflow /app

# Switch to non-root user
USER aiworkflow
WORKDIR /app

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:8080/api/v1/health || exit 1

# Run the application
CMD ["ai-workflow"]

# Gateway runtime stage
FROM debian:bookworm-slim as gateway

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    libpq5 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 gateway

# Copy the gateway binary from builder
COPY --from=builder /app/target/release/graphql-gateway /usr/local/bin/graphql-gateway

# Switch to non-root user
USER gateway
WORKDIR /app

# Expose port
EXPOSE 4000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:4000/health || exit 1

# Run the gateway
CMD ["graphql-gateway"]