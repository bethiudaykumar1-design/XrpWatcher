FROM rust:latest

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy everything
COPY . .

# Build the application
RUN cargo build --release --jobs 2

# Run the binary
CMD ["./target/release/one"]