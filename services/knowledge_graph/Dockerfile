# Multi-stage build for Knowledge Graph service
FROM rust:1.75-alpine AS builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    pkgconfig \
    protobuf-dev

# Create build directory
WORKDIR /build

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY services/knowledge_graph/Cargo.toml ./services/knowledge_graph/
COPY services/shared/Cargo.toml ./services/shared/

# Create dummy files for dependency caching
RUN mkdir -p services/knowledge_graph/src services/shared/src && \
    echo "fn main() {}" > services/knowledge_graph/src/main.rs && \
    echo "fn main() {}" > services/shared/src/lib.rs

# Build dependencies
RUN cargo build --release --package knowledge_graph

# Copy actual source code
COPY services/knowledge_graph/src ./services/knowledge_graph/src
COPY services/shared/src ./services/shared/src

# Build the service with optimizations
RUN touch services/knowledge_graph/src/main.rs && \
    RUSTFLAGS="-C target-cpu=native -C opt-level=3" \
    cargo build --release --package knowledge_graph

# Minimize binary size
RUN strip /build/target/release/knowledge_graph

# Runtime stage using scratch for minimal size
FROM scratch

# Copy necessary files from builder
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /build/target/release/knowledge_graph /knowledge_graph

# DGraph connection will be configured via environment variables
ENV DGRAPH_URL=http://dgraph:9080

# Expose API port
EXPOSE 8083

# Run the service
ENTRYPOINT ["/knowledge_graph"]