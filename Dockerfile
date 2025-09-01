# Use the official Rust image as a builder
FROM rust:1.75 as builder

# Set the working directory
WORKDIR /usr/src/frogolio

# Copy the manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build the dependencies
RUN cargo build --release

# Remove the dummy main.rs and copy the real source code
RUN rm src/main.rs
COPY . .

# Build the application
RUN cargo build --release

# Create a new stage with a minimal runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -r -s /bin/false frogolio

# Create the application directory
WORKDIR /app

# Copy the binary from builder
COPY --from=builder /usr/src/frogolio/target/release/frogolio /app/frogolio

# Create data directory for SQLite database
RUN mkdir -p /app/data && chown -R frogolio:frogolio /app

# Switch to non-root user
USER frogolio

# Expose the port
EXPOSE 3000

# Set environment variables
ENV DATABASE_URL=sqlite:///app/data/frogolio.db
ENV RUST_LOG=frogolio=info

# Run the application
CMD ["./frogolio"] 