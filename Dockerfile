FROM rust:1.77.2-bookworm AS chef 
# We only pay the installation cost once, 
# it will be cached from the second build onwards
RUN cargo install cargo-chef 
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm AS runtime
WORKDIR /app

COPY --from=builder /app/target/release/api /app
COPY --from=builder /app/target/release/daemon /app
COPY --from=builder /app/target/release/parser /app
COPY --from=builder /app/target/release/setup /app
COPY --from=builder /app/target/release/pricer /app
COPY --from=builder /app/target/release/assetor /app
COPY --from=builder /app/target/release/charter /app
COPY --from=builder /app/Settings.toml /app

RUN apt-get update && apt-get install -y openssl ca-certificates libpq-dev postgresql

CMD ["sh", "-c", "/app/${BINARY}"]
