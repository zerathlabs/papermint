---
title: "Print Daemon (papermintd)"
description: "High-performance local HTTP REST sidecar service for web apps and kiosks"
---

# 🌿 Print Daemon (`papermintd`)

`papermintd` is a lightweight native HTTP microservice daemon built on **Tokio** and **Axum**. It runs as a local background service on POS terminals, kiosks, or kitchen display systems, allowing web applications, mobile devices, and browser-based point-of-sale software to print via standard JSON HTTP requests.

---

## Why Use a Print Daemon?

Web browsers (Chrome, Safari, Firefox) run in a sandboxed security model that prevents web pages from opening raw TCP sockets to Port 9100 or claiming raw USB devices.

`papermintd` runs locally (e.g. `http://localhost:8080`) as a **print sidecar**:
1. Your web app or Electron POS makes a simple `fetch('http://localhost:8080/api/print')`.
2. `papermintd` receives the JSON ticket, compiles it into ESC/POS or StarPRNT bytes in ~10μs, and delivers it over USB, TCP, or Serial.
3. Your web app can query real-time sensor telemetry (`paper empty`, `cover open`) via `GET /api/status`.

---

## Running the Daemon

```bash
# Launch with default port (8080)
cargo run --package papermint-daemon

# Launch with custom host and port
PORT=9100 HOST=0.0.0.0 cargo run --package papermint-daemon
```

---

## Complete API Reference

For the full list of endpoints, request models, and curl examples, see the [Daemon REST API Reference](/reference/daemon-api/).
