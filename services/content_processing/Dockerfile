# Multi-stage build for minimal image size
FROM rust:1.75-alpine AS builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    pkgconfig \
    postgresql-dev

# Create app directory
WORKDIR /build

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY services/content_processing/Cargo.toml ./services/content_processing/
COPY services/shared/Cargo.toml ./services/shared/

# Create dummy files for caching dependencies
RUN mkdir -p services/content_processing/src services/shared/src && \
    echo "fn main() {}" > services/content_processing/src/main.rs && \
    echo "fn main() {}" > services/shared/src/lib.rs

# Build dependencies
RUN cargo build --release --package content_processing

# Copy actual source code
COPY services/content_processing/src ./services/content_processing/src
COPY services/shared/src ./services/shared/src

# Build the service
RUN touch services/content_processing/src/main.rs && \
    cargo build --release --package content_processing

# Runtime stage
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    libgcc \
    libssl3 \
    libpq \
    && adduser -D -u 1000 appuser

# Copy binary from builder
COPY --from=builder /build/target/release/content_processing /usr/local/bin/

# Copy migrations
COPY services/content_processing/migrations /app/migrations

# Create necessary directories
RUN mkdir -p /app/data /app/logs && \
    chown -R appuser:appuser /app

# Switch to non-root user
USER appuser
WORKDIR /app

# Expose port
EXPOSE 8082

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8082/health || exit 1

# Run the service
ENTRYPOINT ["/usr/local/bin/content_processing"]