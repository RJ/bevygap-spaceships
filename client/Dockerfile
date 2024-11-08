FROM metabrew/rust-chef-sccache:0.0.6 AS base
ENV RUSTFLAGS=--cfg=web_sys_unstable_apis
ENV CARGO_BUILD_TARGET=wasm32-unknown-unknown
#RUN rustup target add wasm32-unknown-unknown

# cargo build --release --target wasm32-unknown-unknown -p client

# Cargo Chef Planner
FROM base AS planner
WORKDIR /app
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo chef prepare --bin client --recipe-path recipe.json

# Cargo Chef Builder
FROM base AS builder
# Use LLVM clang not default cc, otherwise prone to weird silent wasm linker failures.
# Browser reports: 
#    Uncaught TypeError: Failed to resolve module specifier "env".
#    Relative references must start with either "/", "./", or "../".
#
# and .js file starts with:
#   import * as __wbg_star0 from 'env';
#
# Using clang prevents this.
ENV TARGET_CC=/usr/bin/clang
WORKDIR /app

COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo chef cook --release --bin client --recipe-path recipe.json

# Actually build the application
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo build --release -p client

RUN wasm-bindgen --no-typescript --target web \
    --out-dir /app/www/ \
    --out-name "bevygap-spaceships" \
    ./target/wasm32-unknown-unknown/release/client.wasm

# copy to /usr/share/nginx/html
# Final stage: Create minimal image - this should be an nginx image ready to deploy?
#FROM gcr.io/distroless/cc-debian12
FROM nginx:mainline-alpine
COPY --from=builder /app/www/ /usr/share/nginx/html/
COPY --from=builder /app/assets/ /usr/share/nginx/html/assets/
COPY --from=builder /app/client/www/* /usr/share/nginx/html/

EXPOSE 80
