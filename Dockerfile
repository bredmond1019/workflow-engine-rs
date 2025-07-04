# Multi-stage build for optimized image size
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build dependencies and application
RUN cargo build --release --bin workflow-engine

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