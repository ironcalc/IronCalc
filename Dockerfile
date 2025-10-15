FROM rust:latest AS builder

WORKDIR /app
COPY . .

# Tools + wasm toolchain + Node via nvm
RUN apt-get update && apt-get install -y --no-install-recommends \
      bash curl ca-certificates make \
    && rustup target add wasm32-unknown-unknown \
    && cargo install wasm-pack \
    && bash -lc "curl -fsSL https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.3/install.sh | bash -" \
    && bash -lc '\
         export NVM_DIR="$HOME/.nvm" && \
         source "$NVM_DIR/nvm.sh" && \
         nvm install 22 && nvm alias default 22 && \
         nroot="$NVM_DIR/versions/node/$(nvm version default)/bin" && \
         ln -sf "$nroot/node" /usr/local/bin/node && \
         ln -sf "$nroot/npm"  /usr/local/bin/npm  && \
         ln -sf "$nroot/npx"  /usr/local/bin/npx \
       ' \
    && npm install typescript \
    && rm -rf /var/lib/apt/lists/*

# build the server
RUN cargo build --release --manifest-path webapp/app.ironcalc.com/server/Cargo.toml

# build the wasm
RUN make -C bindings/wasm

# build the widget
WORKDIR /app/webapp/IronCalc
RUN npm install && npm run build

# build the frontend app
WORKDIR /app/webapp/app.ironcalc.com/frontend
RUN npm install && npm run build

# build the xlsx_2_icalc binary (we don't need the release version here)
WORKDIR /app/xlsx
RUN cargo build

WORKDIR /app
# copy the artifacts to a dist/ directory
RUN mkdir dist
RUN mkdir dist/frontend
RUN cp -r webapp/app.ironcalc.com/frontend/dist/* dist/frontend/
RUN mkdir dist/server
RUN cp webapp/app.ironcalc.com/server/target/release/ironcalc_server dist/server/
RUN cp webapp/app.ironcalc.com/server/Rocket.toml dist/server/
RUN cp webapp/app.ironcalc.com/server/ironcalc.sqlite dist/server/

# Create ic files in docs
RUN mkdir -p dist/frontend/models

# Loop over all xlsx files in xlsx/tests/docs & templates and convert them to .ic
RUN bash -lc 'set -euo pipefail; \
  mkdir -p dist/frontend/models; \
  shopt -s nullglob; \
  for xlsx_file in xlsx/tests/docs/*.xlsx; do \
    base_name="${xlsx_file##*/}"; base_name="${base_name%.xlsx}"; \
    ./target/debug/xlsx_2_icalc "$xlsx_file" "dist/frontend/models/${base_name}.ic"; \
  done; \
  for xlsx_file in xlsx/tests/templates/*.xlsx; do \
    base_name="${xlsx_file##*/}"; base_name="${base_name%.xlsx}"; \
    ./target/debug/xlsx_2_icalc "$xlsx_file" "dist/frontend/models/${base_name}.ic"; \
  done'

# ---------- server runtime ----------
FROM debian:bookworm-slim AS server-runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /app
# Copy EVERYTHING you put in dist/server (binary + Rocket.toml + DB)
COPY --from=builder /app/dist/server/ ./
# Make sure Rocket binds to the container IP; explicitly point to the config file
ENV ROCKET_ADDRESS=0.0.0.0 \
    ROCKET_PORT=8000 \
    ROCKET_CONFIG=/app/Rocket.toml
EXPOSE 8000
# Run from /app so relative paths in Rocket.toml/DB work
CMD ["./ironcalc_server"]

# ---------- caddy runtime (serves frontend + reverse-proxy /api) ----------
FROM caddy:latest AS caddy-runtime

WORKDIR /srv

# Copy the frontend build output to /srv
COPY --from=builder /app/dist/frontend/ /srv/

# Copy the Caddyfile
COPY --from=builder /app/webapp/app.ironcalc.com/Caddyfile.compose /etc/caddy/Caddyfile

