FROM rust:1.76-slim-bookworm as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/ducktape
COPY . .

# Build the release binary
RUN cargo build --release

# Create a minimal runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/ducktape/target/release/ducktape /usr/local/bin/
COPY --from=builder /usr/src/ducktape/.env.example /.env.example

# Create a non-root user
RUN useradd -m ducktape
USER ducktape

EXPOSE 3000

ENTRYPOINT ["ducktape"]
CMD ["--api-server"]