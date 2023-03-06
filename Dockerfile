FROM rust:latest AS chef
# We only pay the installation cost once,
# it will be cached from the second build onwards
RUN cargo install cargo-chef

WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo install --path .

# We don't need the Rust toolchain to run the binary!
FROM gcr.io/distroless/cc-debian11

COPY --from=builder /usr/local/cargo/bin/amica-premium-api /app/amica-premium-api
ENV HTTP_BIND="0.0.0.0:80"
CMD ["/app/amica-premium-api"]