# app1/Dockerfile
FROM rust:1.75 AS builder

WORKDIR /usr/src/app
COPY . .

# Build your app
RUN cargo build --release

# Final stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/your-app-name /usr/local/bin/

# Run your app
CMD ["your-app-name"]