# syntax=docker/dockerfile:1

# ---------- Frontend: Rust -> WebAssembly ----------
FROM rust:1.98-bookworm AS frontend

RUN rustup target add wasm32-unknown-unknown

ARG WASM_PACK_VERSION=0.15.0
RUN set -eux; \
    case "$(uname -m)" in \
      aarch64) triple="aarch64-unknown-linux-musl" ;; \
      x86_64) triple="x86_64-unknown-linux-musl" ;; \
      *) echo "arquitetura não suportada: $(uname -m)" >&2; exit 1 ;; \
    esac; \
    curl -fsSL "https://github.com/wasm-bindgen/wasm-pack/releases/download/v${WASM_PACK_VERSION}/wasm-pack-v${WASM_PACK_VERSION}-${triple}.tar.gz" -o /tmp/wasm-pack.tar.gz; \
    tar -xzf /tmp/wasm-pack.tar.gz -C /tmp; \
    cp "/tmp/wasm-pack-v${WASM_PACK_VERSION}-${triple}/wasm-pack" /usr/local/bin/wasm-pack; \
    chmod +x /usr/local/bin/wasm-pack; \
    wasm-pack --version

WORKDIR /app/frontend
COPY frontend/Cargo.toml frontend/Cargo.lock ./
COPY frontend/LICENSE-MIT frontend/LICENSE-APACHE ./
COPY frontend/src ./src
COPY frontend/index.html ./
COPY frontend/config.js ./
COPY frontend/styles ./styles
COPY frontend/fonts ./fonts

RUN wasm-pack build --target web --release --no-opt --out-dir dist/pkg
RUN cp index.html config.js dist/ && cp -R styles dist/styles && cp -R fonts dist/fonts
COPY design-system /app/design-system
RUN mkdir -p dist/design-system && cp -R /app/design-system/. dist/design-system/

# ---------- Backend: Rust nativo ----------
FROM rust:1.98-bookworm AS backend

WORKDIR /app
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
COPY backend/data ./data
RUN cargo build --release

# ---------- Runtime ----------
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=backend /app/target/release/mirante-backend /usr/local/bin/mirante
COPY --from=frontend /app/frontend/dist /app/dist

RUN mkdir -p /data

ENV MIRANTE_PORT=8091 \
    MIRANTE_DIST=/app/dist \
    MIRANTE_DB=/data/mirante.db \
    MIRANTE_CACHE_TTL=1800 \
    RUST_LOG=info

EXPOSE 8091

HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
  CMD curl -fsS http://127.0.0.1:8091/api/health || exit 1

CMD ["mirante"]
