# Build stage
FROM rust:1.78 as builder

WORKDIR /app

# Install dependencies for grammers
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock* ./

# Create dummy main to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy actual source
COPY src ./src
COPY config.example.toml ./config.toml

# Build application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/railway-monitor /app/railway-monitor
COPY config.example.toml /app/config.toml

# Create non-root user
RUN useradd -r -s /bin/false appuser && chown -R appuser:appuser /app
USER appuser

# Expose port (not needed for worker but good practice)
EXPOSE 8080

# Run
CMD ["/app/railway-monitor"]