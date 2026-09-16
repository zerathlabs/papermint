---
title: Cloud & Remote WebSocket Printing
description: Push print jobs from cloud web dashboards (Next.js, Node.js, Shopify) to in-store countertop thermal printers through firewalls and NAT without port forwarding.
---

A common hurdle in modern cloud Point of Sale (POS) and online ordering systems is **how a cloud server pushes print jobs to physical printers inside retail stores and restaurants**.

Retail store countertop printers sit behind ISP NAT routers with dynamic private IP addresses (`192.168.1.x`). Inbound connections from the public Internet are blocked by default, and setting up dynamic DNS or port forwarding is dangerous, fragile, and hard to maintain across multiple locations.

---

## The Outbound WebSocket Gateway Architecture

`papermint` solves this problem with the **Outbound WebSocket Gateway Pattern**. Instead of your cloud server trying to reach inside the store, the in-store daemon (`papermintd`) maintains a persistent **outbound** encrypted WebSocket connection to your cloud backend:

```
┌─────────────────────────────────────────────────────────────┐
│ Cloud Server (VPS / Cloudflare / Hono / Node.js / Next.js)  │
│  • Customer places order online / Barista dashboard         │
│  • Generates receipt ticket or label JSON payload           │
│  • Dispatches PRINT_JOB to shop WebSocket channel           │
└──────────────────────────────┬──────────────────────────────┘
                               │ Persistent Outbound WSS
                               ▼ (Bypasses NAT & Firewalls!)
┌─────────────────────────────────────────────────────────────┐
│ In-Store Gateway (`papermintd` on Raspberry Pi / PC / Mac)  │
│  • Connects to wss://... on boot with bearer authentication │
│  • Listens for PRINT_JOB events                             │
│  • Formats wire bytes in ~10µs via `papermint`              │
│  • Streams to physical USB, TCP, or Serial printer          │
│  • Replies with instant ACK and paper sensor telemetry     │
└──────────────────────────────┬──────────────────────────────┘
                               │ Raw Wire Bytes
                               ▼
 ┌───────────────────────────────────────────────────────────┐
 │ Countertop Receipt / Kitchen Printer (Port 9100 / USB)    │
 └───────────────────────────────────────────────────────────┘
```

---

## 1. Running the In-Store Gateway (`papermintd`)

The `papermintd` sidecar daemon supports outbound gateway mode out of the box with zero extra software:

```bash
# Connect to your cloud server with shop authentication:
papermintd \
  --gateway "wss://pos-api.yourbrand.com/ws/printer/shop_042" \
  --token "sk_live_9b4e78a2c1f" \
  --shop-id "shop_042" \
  --printer "tcp://192.168.1.200:9100"
```

### Supported `--printer` Formats

- **Network / LAN / WiFi**: `tcp://192.168.1.200:9100` or `192.168.1.200:9100`
- **USB**: `usb:04b8:0202` (VID:PID in hex, e.g. Epson TM-T88VI)
- **Serial**: `serial:/dev/ttyUSB0:19200`
- **Mock**: `mock` (for virtual testing in CI/CD)

### Environment Variable Alternative

You can configure `papermintd` using environment variables or a `.env` file:

```bash
GATEWAY_URL="wss://pos-api.yourbrand.com/ws/printer/shop_042"
GATEWAY_TOKEN="sk_live_9b4e78a2c1f"
SHOP_ID="shop_042"
DEFAULT_PRINTER="tcp://192.168.1.200:9100"
PORT=8080
```

> [!TIP]
> While running in gateway mode, `papermintd` simultaneously serves its local HTTP REST API on `http://127.0.0.1:8080`. Local cash registers and tablets can still query `/health` or send local receipts over LAN at the same time.

---

## 2. Cloud Server Implementation (Node.js / Hono / Bun)

Here is a minimal cloud WebSocket server that registers in-store printers and pushes jobs when web orders arrive:

```typescript
import { WebSocketServer, WebSocket } from "ws";

const wss = new WebSocketServer({ port: 3000 });
const storeSockets = new Map<string, WebSocket>();

wss.on("connection", (ws, req) => {
  let authenticatedShop: string | null = null;

  ws.on("message", (raw) => {
    const msg = JSON.parse(raw.toString());

    // 1. In-store daemon registers upon connecting
    if (msg.type === "register") {
      if (isValidToken(msg.token)) {
        authenticatedShop = msg.shop_id;
        storeSockets.set(msg.shop_id, ws);
        console.log(`✅ Shop ${msg.shop_id} connected (v${msg.version})`);
      } else {
        ws.close(4001, "Unauthorized");
      }
    }

    // 2. In-store daemon confirms print completion
    if (msg.type === "ack") {
      console.log(` Job ${msg.job_id} acknowledged (success: ${msg.success})`);
    }
  });

  ws.on("close", () => {
    if (authenticatedShop) storeSockets.delete(authenticatedShop);
  });
});

// Function to push a receipt to a specific shop from Next.js / API route:
export function dispatchCloudReceipt(shopId: string, order: any) {
  const ws = storeSockets.get(shopId);
  if (!ws || ws.readyState !== WebSocket.OPEN) {
    throw new Error(`Shop ${shopId} is currently offline`);
  }

  const job = {
    type: "print_job",
    job_id: `ord_${order.id}`,
    dialect: "escpos",
    paper_width: "80mm",
    ticket: {
      title: "MINT SPECIALTY COFFEE",
      subtitle: `Order #${order.number}`,
      items: order.items.map((i: any) => ({
        qty: `${i.qty}x`,
        name: i.name,
        price: `$${i.price.toFixed(2)}`,
        total: `$${(i.qty * i.price).toFixed(2)}`,
      })),
      totals: [
        { label: "SUBTOTAL", value: `$${order.subtotal.toFixed(2)}` },
        { label: "TAX (8%)", value: `$${order.tax.toFixed(2)}` },
        { label: "TOTAL", value: `$${order.total.toFixed(2)}` },
      ],
      qr: `https://brand.com/receipt/${order.id}`,
      cut_mode: "full",
      beep: 2,
    },
  };

  ws.send(JSON.stringify(job));
}
```

---

## 3. Wire Protocol Specification

### Cloud $\to$ Gateway (`print_job`)

```json
{
  "type": "print_job",
  "job_id": "job_01h8x9p3",
  "dialect": "escpos",
  "paper_width": "80mm",
  "ticket": {
    "title": "STORE TAKEOUT",
    "items": [
      { "qty": "2x", "name": "Latte", "total": "$9.00" }
    ],
    "cut_mode": "full"
  }
}
```

### Gateway $\to$ Cloud (`ack`)

```json
{
  "type": "ack",
  "job_id": "job_01h8x9p3",
  "success": true,
  "bytes_sent": 412,
  "error": null
}
```

### Heartbeat & Telemetry

- **Ping / Pong**: Send `{"type": "ping", "timestamp": 1726400000}`. Gateway immediately replies with `{"type": "pong"}`.
- **Status Query**: Send `{"type": "status_query"}`. Gateway replies with real-time sensor status (`is_online`, `is_ready`, `paper: "Adequate"`, `cover: "Closed"`).

---

## 4. Production Systemd Service (Raspberry Pi / Linux)

To run `papermintd` as an unkillable system service in retail stores:

Create `/etc/systemd/system/papermintd.service`:

```ini
[Unit]
Description=papermint Thermal Print Gateway Sidecar
After=network.target

[Service]
Type=simple
User=pi
Environment=GATEWAY_URL=wss://pos-api.yourbrand.com/ws/printer/shop_042
Environment=GATEWAY_TOKEN=sk_live_9b4e78a2c1f
Environment=SHOP_ID=shop_042
Environment=DEFAULT_PRINTER=tcp://192.168.1.200:9100
ExecStart=/usr/local/bin/papermintd
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
```

Enable and start the service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now papermintd
```

