# Caminho relativo: Dockerfile
# Build reprodutível do dotree (ADR 0001, ADR 0002, spec ENT-04):
#   1. Rust -> WebAssembly (núcleo)   2. Node -> site estático   3. servidor estático, sem ferramentas de build
# Uso:  docker build -t dotree .  &&  docker run --rm -p 8080:8080 dotree
# BASE_PATH vazio serve na raiz; o GitHub Pages usa /dotree (definido no CI, não aqui).

# ---- 1. núcleo em WebAssembly ----
FROM rust:1.97-slim AS wasm
ARG WASM_PACK_VERSION=0.15.0
RUN rustup target add wasm32-unknown-unknown \
 && cargo install wasm-pack --version "${WASM_PACK_VERSION}" --locked
WORKDIR /src
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY app/core app/core
COPY app/wasm app/wasm
RUN wasm-pack build app/wasm --release --target web \
      --out-dir ../web/src/lib/wasm --out-name dotree --no-pack

# ---- 2. site estático ----
FROM node:25-slim AS web
ARG BASE_PATH=""
ENV BASE_PATH=${BASE_PATH} CI=1
RUN corepack enable
WORKDIR /src/app/web
COPY app/web/package.json app/web/pnpm-lock.yaml app/web/pnpm-workspace.yaml app/web/.npmrc ./
RUN pnpm install --frozen-lockfile
COPY app/web/ ./
COPY --from=wasm /src/app/web/src/lib/wasm ./src/lib/wasm
RUN pnpm run build:web

# ---- 3. servidor estático (porta 8080, usuário sem privilégio) ----
FROM nginxinc/nginx-unprivileged:stable-alpine AS site
COPY --from=web /src/app/web/build /usr/share/nginx/html
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s CMD wget -qO- http://127.0.0.1:8080/ >/dev/null || exit 1
