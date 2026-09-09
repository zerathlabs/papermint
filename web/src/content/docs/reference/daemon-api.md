---
title: "Daemon REST API"
description: "Complete HTTP REST API specification for papermintd"
---

# 🌿 `papermintd` HTTP REST API Specification

`papermintd` is a lightweight native HTTP microservice daemon built on **Tokio** and **Axum**. It allows web applications, mobile devices, local Electron POS systems, and microservices to print receipts, check real-time sensor status, and trigger cash drawers via standard JSON HTTP requests.

---

## 1. Running the Daemon

Launch the daemon locally:

```bash
# Run with default port (8080)
cargo run --package papermint-daemon

# Run on a custom port and host
PORT=9101 HOST=0.0.0.0 cargo run --package papermint-daemon
```

Output:
```text
🌿 papermintd daemon listening on http://127.0.0.1:8080
```

---

## 2. API Endpoints

### 2.1 Health Check: `GET /health`

Returns daemon health and uptime.

* **Request**: `GET /health`
* **Response (`200 OK`)**:
  ```json
  {
    "status": "ok",
    "version": "0.1.0",
    "uptime_secs": 42
  }
  ```

---

### 2.2 Enumerate Devices: `GET /api/devices`

Discovers connected USB Class 7 printers and available serial/COM ports.

* **Request**: `GET /api/devices`
* **Response (`200 OK`)**:
  ```json
  {
    "serial_ports": ["/dev/ttyUSB0", "/dev/ttyS0"],
    "usb_printers": [
      {
        "vendor_id": 1208,
        "product_id": 514,
        "serial_number": "12345678",
        "manufacturer": "EPSON",
        "product_name": "TM-T20III"
      }
    ]
  }
  ```

---

### 2.3 Print Receipt: `POST /api/print`

Prints a structured JSON ticket or raw binary wire bytes to a target printer.

#### Targets:
* **TCP**: `{"type": "tcp", "address": "192.168.1.200:9100"}`
* **USB**: `{"type": "usb", "vendor_id": 1208, "product_id": 514}`
* **Serial**: `{"type": "serial", "port": "/dev/ttyUSB0", "baud": 115200}`
* **Mock**: `{"type": "mock"}` (Returns simulated receipt preview in ASCII)

#### Dialects:
* `"escpos"` (Epson, Bixolon, Citizen, Xprinter)
* `"star"` (StarPRNT TSP100, TSP650II, mC-Print)

#### Example 1: Printing a Structured Restaurant Ticket

```bash
curl -X POST http://localhost:8080/api/print \
  -H "Content-Type: application/json" \
  -d '{
    "target": {"type": "mock"},
    "dialect": "escpos",
    "paper_width": "80mm",
    "ticket": {
      "title": "MINT BISTRO",
      "subtitle": "Table #4 - Order #1042",
      "address": "123 Market St, San Francisco",
      "metadata": [
        {"label": "Server", "value": "Luffy"},
        {"label": "Time", "value": "17:30"}
      ],
      "items": [
        {"qty": "2", "description": "Smashburger", "price": "$9.50", "total": "$19.00"},
        {"qty": "1", "description": "Truffle Fries", "price": "$5.00", "total": "$5.00"},
        {"qty": "2", "description": "Mint Iced Tea", "price": "$3.50", "total": "$7.00"}
      ],
      "totals": [
        {"label": "Subtotal", "value": "$31.00"},
        {"label": "Tax (8.5%)", "value": "$2.64"},
        {"label": "TOTAL", "value": "$33.64"}
      ],
      "qr": "https://mintbistro.com/pay/1042",
      "barcode": "10423364",
      "footer": "Thank you! Please visit us again.",
      "cut_mode": "full",
      "open_drawer": true,
      "beep": 1
    }
  }'
```

* **Response (`200 OK`)**:
  ```json
  {
    "success": true,
    "bytes_sent": 384,
    "mock_preview": "      MINT BISTRO      \nTable #4 - Order #1042\n..."
  }
  ```

#### Example 2: Sending Raw Wire Bytes

```bash
curl -X POST http://localhost:8080/api/print \
  -H "Content-Type: application/json" \
  -d '{
    "target": {"type": "tcp", "address": "192.168.1.200:9100"},
    "dialect": "escpos",
    "raw_bytes": [27, 64, 72, 101, 108, 108, 111, 10, 29, 86, 0]
  }'
```

---

### 2.4 Query Hardware Sensor Telemetry: `POST /api/status`

Queries real-time printer sensors (paper roll, cover open, drawer state, head overheat, and cutter jam).

* **Request**:
  ```bash
  curl -X POST http://localhost:8080/api/status \
    -H "Content-Type: application/json" \
    -d '{
      "target": {"type": "tcp", "address": "192.168.1.200:9100"},
      "dialect": "escpos"
    }'
  ```

* **Response (`200 OK`)**:
  ```json
  {
    "is_online": true,
    "is_ready": true,
    "paper": "Adequate",
    "cover": "Closed",
    "drawer": "Closed",
    "cutter_error": false,
    "head_overheated": false
  }
  ```

---

### 2.5 Trigger Cash Drawer: `POST /api/drawer`

Sends an electrical solenoid kick pulse to open the cash drawer.

* **Request**:
  ```bash
  curl -X POST http://localhost:8080/api/drawer \
    -H "Content-Type: application/json" \
    -d '{
      "target": {"type": "tcp", "address": "192.168.1.200:9100"},
      "dialect": "escpos",
      "pin": 2
    }'
  ```

* **Response (`200 OK`)**:
  ```json
  {
    "success": true,
    "message": "Cash drawer kick command sent to Pin 2"
  }
  ```
