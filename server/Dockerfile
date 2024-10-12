# Stage 1: Install sccache with caching
FROM lukemathwalker/cargo-chef:latest-rust-1 AS sccache-installer
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo install sccache --version ^0.8.2
ENV RUSTC_WRAPPER=sccache SCCACHE_DIR=/sccache

# Stage 2: Cargo Chef Planner
FROM sccache-installer AS planner
WORKDIR /app
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo chef prepare --bin server --recipe-path recipe.json

# Stage 3: Cargo Chef Builder
FROM sccache-installer AS builder
WORKDIR /app

RUN --mount=type=cache,target=/var/lib/apt/lists,type=cache,sharing=locked \
    --mount=type=cache,target=/var/cache/apt,type=cache,sharing=locked \
    apt-get update && apt-get -y --no-install-recommends install libudev-dev

COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo chef cook --release --bin server --recipe-path recipe.json

# Stage 4: Actually build the application
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