FROM ghcr.io/fermyon/spin:v2.7.0

WORKDIR /app

# Copy Spin configuration, web tag bundle, and compiled WASM component
COPY spin.toml /app/spin.toml
COPY static/t.min.js /app/static/t.min.js
COPY target/wasm32-wasip1/release/beacon_server-opt.wasm /app/target/wasm32-wasip1/release/beacon_server.wasm

EXPOSE 8080

ENTRYPOINT ["/bin/sh", "-c", "mkdir -p /var/log/beacon && exec spin up --listen 0.0.0.0:8080 --file /app/spin.toml 2>&1 | tee -a /var/log/beacon/events.log"]
