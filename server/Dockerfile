FROM metabrew/rust-chef-sccache:latest AS base
ENV RUSTC_WRAPPER=sccache SCCACHE_DIR=/sccache

# Cargo Chef Planner
FROM base AS planner
WORKDIR /app
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo chef prepare --bin server --recipe-path recipe.json

# Cargo Chef Builder
FROM base AS builder
WORKDIR /app

COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo chef cook --release --bin server --recipe-path recipe.json

# Actually build the application
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo build --release -p server

# Final stage: Create minimal image
FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/assets/ /app/assets/
COPY --from=builder /app/certificates/ /app/certificates/
COPY --from=builder /app/target/release/server /app/
WORKDIR /app
ENTRYPOINT ["./server"]
EXPOSE 6420/udp