# Multi-stage build for minimal WebSocket service image
FROM rust:1.75-alpine AS builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    pkgconfig

# Create build directory
WORKDIR /build

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY services/realtime_communication/Cargo.toml ./services/realtime_communication/
COPY services/shared/Cargo.toml ./services/shared/

# Create dummy files for dependency caching
RUN mkdir -p services/realtime_communication/src services/shared/src && \
    echo "fn main() {}" > services/realtime_communication/src/main.rs && \
    echo "fn main() {}" > services/shared/src/lib.rs

# Build dependencies
RUN cargo build --release --package realtime_communication

# Copy actual source code
COPY services/realtime_communication/src ./services/realtime_communication/src
COPY services/shared/src ./services/shared/src

# Build the service
RUN touch services/realtime_communication/src/main.rs && \
    cargo build --release --package realtime_communication

# Runtime stage - using distroless for minimal attack surface
FROM gcr.io/distroless/cc-debian12

# Copy binary from builder
COPY --from=builder /build/target/release/realtime_communication /app/realtime

# Create non-root user directories
USER nonroot:nonroot

# Expose WebSocket port
EXPOSE 8081

# Health check endpoint is at /health
ENV HEALTH_CHECK_URL=http://localhost:8081/health

# Run the service
ENTRYPOINT ["/app/realtime"]