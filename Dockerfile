FROM node:24-alpine AS builder
WORKDIR /app
RUN apk add --no-cache curl build-base python3
RUN curl https://sh.rustup.rs -sSf | sh -s - -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN cargo install wasm-pack
COPY . .
RUN cd /app/bindings/nodejs && npm install && npm run build
RUN npm uninstall tsc && npm install -D typescript && cd /app/bindings/wasm && make
RUN cd /app/webapp/IronCalc && npm install && npm run build
RUN cd /app/webapp/app.ironcalc.com/frontend && npm install && npm run build
RUN cd /app/webapp/app.ironcalc.com/server && cargo build --release


FROM caddy:2.10.0-alpine

RUN apk add build-base musl-dev
COPY --from=builder /app/webapp/app.ironcalc.com/frontend/dist /usr/share/nginx/html
COPY --from=builder /app/webapp/app.ironcalc.com/server/target/release/ironcalc_server /usr/local/bin/ironcalc_server
COPY webapp/app.ironcalc.com/Caddyfile /etc/caddy/Caddyfile
COPY webapp/app.ironcalc.com/server/Rocket.toml .

EXPOSE 2080
