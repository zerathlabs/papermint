# 🌐 papermintd — Local HTTP Print Daemon

`papermintd` is a standalone, high-performance background HTTP REST server for thermal receipt printers, powered by **Axum** and **Tokio**.

---

## Why does this exist?

Web browsers (Chrome, Safari, Firefox, Edge) operate inside a security sandbox and **cannot directly talk to local USB thermal printers or serial ports**.

If you are building a **web-based POS, kiosk, or cloud dashboard** (e.g. Next.js, React, Vue, PHP, Django, Shopify POS), your web frontend cannot send raw binary ESC/POS bytes over a USB cable.

With `papermintd`:
1. `papermintd` runs locally in the background on the cashier's computer or kiosk tablet (on port `8080`).
2. Your web app simply calls standard HTTP `fetch`:
   ```javascript
   await fetch('http://localhost:8080/print', {
     method: 'POST',
     headers: { 'Content-Type': 'application/json' },
     body: JSON.stringify({
       transport: 'usb',
       dialect: 'escpos',
       receipt: { ... }
     })
   });
   ```
3. The daemon communicates with the hardware printer (USB, Network IP, or Serial) and cuts the receipt instantly.

---

## Running the Daemon

### From Source:
```bash
cargo run --package papermint-daemon --release
```

### From Pre-Built Release Binaries:
Pre-compiled standalone binaries for **Linux**, **macOS**, and **Windows** are attached to every [GitHub Release](https://github.com/zerathlabs/papermint/releases).

---

## REST API Endpoints

- `GET /health` — Service liveness check.
- `GET /devices` — List detected USB thermal printers and serial ports.
- `POST /print` — Submit a JSON receipt payload to be printed to USB, TCP, or Serial.
- `POST /status` — Query real-time printer hardware status (paper out, cover open, etc.).

Full API documentation: [https://papermint.zerathlabs.com/reference/daemon-api/](https://papermint.zerathlabs.com/reference/daemon-api/)
