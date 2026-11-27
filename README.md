# System_dev
# NGINX + Rust (Axum) Reverse Proxy Lab

Local learning project showing how **NGINX** acts as an event-driven reverse proxy and round-robin load balancer in front of a **Rust (Axum)** backend service.

## Architecture

```text
Client (curl / browser)
       │
       ▼
NGINX (:8080)  ──▶ sets Host, X-Real-IP, X-Forwarded-For, X-Forwarded-Proto
       │
       ├──▶ Rust Backend Instance #1 (127.0.0.1:3001)
       └──▶ Rust Backend Instance #2 (127.0.0.1:3002)
```

- **`src/main.rs`**: Entry point that starts the server by calling helpers in `src/server.rs`.
- **`src/server.rs`**: Reads `PORT` (default `3001`), wraps `AppState` in `Arc`, configures Axum routes (`/api/info` and `/health`), binds `127.0.0.1:<port>`, and extracts request headers into JSON.
- **`nginx.conf`**: Local reverse-proxy configuration listening on `:8080`, forwarding `/api/` across `127.0.0.1:3001` and `127.0.0.1:3002`, and keeping PID, logs, and temp paths local for non-root execution.

## Project Layout

```text
.
├── Cargo.toml
├── Cargo.lock
├── .gitignore
├── nginx.conf
├── README.md
└── src/
    ├── main.rs
    └── server.rs
```

## Setup and Run

### 1. Start Two Rust Instances

Terminal 1:
```bash
PORT=3001 cargo run
```

Terminal 2:
```bash
PORT=3002 cargo run
```

### 2. Create Temp Directory and Start NGINX

Create `nginx_temp` first so NGINX can create its subdirectory buffers without `No such file or directory`:

```bash
mkdir -p nginx_temp
nginx -p "$PWD" -c "$PWD/nginx.conf" -t
nginx -p "$PWD" -c "$PWD/nginx.conf"
```

Control NGINX from the project root:
```bash
# Reload after config edits
nginx -p "$PWD" -c "$PWD/nginx.conf" -t && nginx -p "$PWD" -c "$PWD/nginx.conf" -s reload

# Stop NGINX cleanly
nginx -p "$PWD" -c "$PWD/nginx.conf" -s quit
```

## Verification

```bash
# Round-robin load balancing across ports 3001 and 3002
for i in 1 2 3 4; do curl -s [http://127.0.0.1:8080/api/info](http://127.0.0.1:8080/api/info); echo; done

# Pre-existing X-Forwarded-For hop chain
curl -s -H "X-Forwarded-For: 203.0.113.10" [http://127.0.0.1:8080/api/info](http://127.0.0.1:8080/api/info)

# Direct backend vs. unmatched path on NGINX
curl -s [http://127.0.0.1:3001/api/info](http://127.0.0.1:3001/api/info)
curl -i -s [http://127.0.0.1:8080/](http://127.0.0.1:8080/)

# Watch logs
tail -f access.log error.log
```

## Notes

- NGINX does not forward `X-Real-IP` or `X-Forwarded-For` unless configured with `proxy_set_header`.
- `X-Real-IP` holds a single IP (`$remote_addr`), while `X-Forwarded-For` appends hops (`$proxy_add_x_forwarded_for`).
- `.with_state(state)` supplies `Arc<AppState>` so Axum produces `Router<()>` while cloning only an atomic reference count per request.