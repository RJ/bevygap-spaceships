## see https://depot.dev/blog/rust-dockerfile-best-practices
##
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
RUN cargo install sccache --version ^0.8.2
ENV RUSTC_WRAPPER=sccache SCCACHE_DIR=/sccache

FROM chef AS planner
WORKDIR /app
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo chef prepare --bin server --recipe-path recipe.json

FROM chef AS builder 
WORKDIR /app
# Cached apt-get install deps
RUN --mount=target=/var/lib/apt/lists,type=cache,sharing=locked \
    --mount=target=/var/cache/apt,type=cache,sharing=locked \
    rm -f /etc/apt/apt.conf.d/docker-clean \
    && apt-get update \
    && apt-get -y --no-install-recommends install libudev-dev
# Cargo chef cooking
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo chef cook --release --bin server --recipe-path recipe.json
# Actually build our application
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo build --release -p server


# the :debug version includes a busybox shell
# FROM gcr.io/distroless/cc-debian12:debug
FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/server /app/
COPY --from=builder /app/assets/ /app/assets/
COPY --from=builder /app/certificates/ /app/certificates/

WORKDIR /app

ENTRYPOINT ["./server"]

# documents but doesn't automatically expose:
EXPOSE 5420/udp
