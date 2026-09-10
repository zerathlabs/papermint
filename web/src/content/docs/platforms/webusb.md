---
title: Direct Browser Printing (WebUSB)
description: Zero-install direct thermal receipt printing from Google Chrome and Microsoft Edge using the WebUSB standard.
---

With **WebUSB**, web applications (Next.js, React, Vue, Svelte, or vanilla JavaScript) can communicate directly with USB thermal receipt printers from the browser—**without installing local print daemons, native desktop drivers, or browser extensions**.

This enables true zero-setup Cloud POS checkout terminals where cashiers simply plug in a USB thermal printer, open your web app in Google Chrome or Microsoft Edge, grant one-time permission, and start printing receipts.

---

## Browser Support & Requirements

| Browser / Environment | Supported? | Requirement |
| :--- | :--- | :--- |
| **Google Chrome (Desktop)** | ✅ Full Support | Chrome 61+ (HTTPS or localhost) |
| **Microsoft Edge (Desktop)** | ✅ Full Support | Edge 79+ (Chromium) |
| **Chrome for Android** | ✅ Full Support | OTG USB cable + WebUSB |
| **Safari / WebKit** | ❌ No | Apple does not implement WebUSB |
| **Mozilla Firefox** | ❌ No | Mozilla does not implement WebUSB |

:::tip[Production Tip]
For environments running on Apple Safari, iOS iPads, or Firefox, use the lightweight **[papermintd print daemon](/platforms/daemon)** or **[papermint-mobile Expo module](/platforms/mobile)**. For Chromium browsers, WebUSB provides the ultimate zero-install user experience.
:::

---

## How WebUSB Thermal Printing Works

Thermal receipt printers identify themselves on the USB bus under **USB Device Class `0x07` (Printers)**:

1. **Device Selection**: The browser invokes `navigator.usb.requestDevice()` to prompt the user to choose their connected receipt printer.
2. **Interface Claiming**: WebUSB opens the device session and claims the interface controlling the printer (`claimInterface()`).
3. **Endpoint Discovery**: The driver locates the active **Bulk OUT** endpoint (the hardware channel that receives raw ESC/POS or StarPRNT bytes).
4. **Binary Transmission**: The application passes the byte array compiled by `papermint` directly into `device.transferOut(endpointNumber, rawBytes)`.

---

## Ready-to-Use React / Next.js Hook

Here is a production-grade, reusable React hook that manages device connection state, endpoint resolution, chunking for large prints, and error handling:

```tsx
// useWebUsbPrinter.ts
import { useState, useCallback } from 'react';

export interface WebUsbPrinterState {
  isSupported: boolean;
  isConnected: boolean;
  deviceName: string | null;
  connect: () => Promise<void>;
  disconnect: () => Promise<void>;
  print: (data: Uint8Array | ArrayBuffer) => Promise<void>;
  error: string | null;
}

export function useWebUsbPrinter(): WebUsbPrinterState {
  const isSupported = typeof navigator !== 'undefined' && 'usb' in navigator;
  const [device, setDevice] = useState<USBDevice | null>(null);
  const [endpointNumber, setEndpointNumber] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);

  const connect = useCallback(async () => {
    setError(null);
    try {
      if (!isSupported) {
        throw new Error('WebUSB is not supported in this browser. Please use Chrome or Edge.');
      }

      // Request any USB device with Printer Class (0x07) or prompt all connected USB devices
      const selectedDevice = await navigator.usb.requestDevice({
        filters: [
          { classCode: 0x07 }, // USB Printer Class
        ],
      });

      await selectedDevice.open();

      // Select active USB configuration (usually config 1)
      if (selectedDevice.configuration === null) {
        await selectedDevice.selectConfiguration(1);
      }

      // Find the interface and Bulk OUT endpoint
      let outEndpoint: USBEndpoint | null = null;
      let targetInterface: USBInterface | null = null;

      for (const iface of selectedDevice.configuration.interfaces) {
        for (const alternate of iface.alternates) {
          const ep = alternate.endpoints.find(
            (e) => e.direction === 'out' && e.type === 'bulk'
          );
          if (ep) {
            outEndpoint = ep;
            targetInterface = iface;
            break;
          }
        }
        if (outEndpoint) break;
      }

      if (!targetInterface || !outEndpoint) {
        throw new Error('Could not find a valid Bulk OUT endpoint on this USB printer.');
      }

      await selectedDevice.claimInterface(targetInterface.interfaceNumber);

      setDevice(selectedDevice);
      setEndpointNumber(outEndpoint.endpointNumber);
    } catch (err: any) {
      setError(err.message || 'Failed to connect to USB printer.');
      throw err;
    }
  }, [isSupported]);

  const disconnect = useCallback(async () => {
    if (device) {
      try {
        await device.close();
      } catch {
        // ignore closure errors
      }
      setDevice(null);
      setEndpointNumber(null);
    }
  }, [device]);

  const print = useCallback(
    async (data: Uint8Array | ArrayBuffer) => {
      setError(null);
      if (!device || endpointNumber === null) {
        throw new Error('Printer is not connected. Call connect() first.');
      }

      const buffer = data instanceof Uint8Array ? data : new Uint8Array(data);
      const CHUNK_SIZE = 16384; // 16 KB safe chunk size

      try {
        for (let offset = 0; offset < buffer.length; offset += CHUNK_SIZE) {
          const slice = buffer.slice(offset, offset + CHUNK_SIZE);
          const result = await device.transferOut(endpointNumber, slice);
          if (result.status !== 'ok') {
            throw new Error(`USB transfer failed with status: ${result.status}`);
          }
        }
      } catch (err: any) {
        setError(err.message || 'Failed to transfer print data.');
        throw err;
      }
    },
    [device, endpointNumber]
  );

  return {
    isSupported,
    isConnected: !!device,
    deviceName: device ? device.productName || 'USB Thermal Printer' : null,
    connect,
    disconnect,
    print,
    error,
  };
}
```

---

## End-to-End Example: POS Checkout in Next.js

In this workflow:
1. Your Node.js / Hono backend or client compiles a receipt via `papermint`.
2. The browser receives the binary buffer (`receipt.encode('escpos')`).
3. `useWebUsbPrinter().print(escposBytes)` shoots the bytes directly into the hardware over USB!

```tsx
// pages/pos-checkout.tsx
import React from 'react';
import { useWebUsbPrinter } from '../hooks/useWebUsbPrinter';

export default function CheckoutPage() {
  const { isSupported, isConnected, deviceName, connect, disconnect, print, error } =
    useWebUsbPrinter();

  async function handlePrintReceipt() {
    // 1. Fetch pre-compiled ESC/POS bytes from your backend API
    const res = await fetch('/api/orders/1042/print-bytes', { method: 'POST' });
    const arrayBuffer = await res.arrayBuffer();

    // 2. Transmit directly to the thermal printer over WebUSB
    await print(new Uint8Array(arrayBuffer));
  }

  if (!isSupported) {
    return <div>WebUSB is not supported. Please use Google Chrome or Microsoft Edge.</div>;
  }

  return (
    <div className="p-6 max-w-md mx-auto space-y-4">
      <h2 className="text-xl font-bold">Cloud POS Checkout</h2>

      {!isConnected ? (
        <button
          onClick={connect}
          className="w-full bg-emerald-600 text-white font-semibold py-2 px-4 rounded"
        >
          Connect USB Thermal Printer
        </button>
      ) : (
        <div className="space-y-3">
          <div className="text-sm text-gray-600">
            Connected: <span className="font-semibold text-gray-900">{deviceName}</span>
          </div>

          <button
            onClick={handlePrintReceipt}
            className="w-full bg-blue-600 text-white font-bold py-3 px-4 rounded shadow"
          >
            🖨️ Print Order Receipt
          </button>

          <button
            onClick={disconnect}
            className="text-xs text-red-500 underline"
          >
            Disconnect Printer
          </button>
        </div>
      )}

      {error && <div className="text-sm text-red-600 font-medium">{error}</div>}
    </div>
  );
}
```

---

## Troubleshooting Windows USB Drivers (WinUSB)

On **Windows**, thermal printers install by default with vendor spooler drivers (e.g. POS58 / POS80 driver), which blocks raw WebUSB access.

To allow Chrome or Edge to talk directly to the printer via WebUSB on Windows:
1. Download **[Zadig](https://zadig.akeo.ie/)** (open-source USB driver utility).
2. Plug in your thermal printer.
3. Open Zadig, click **Options** → **List All Devices**.
4. Select your thermal printer from the dropdown list.
5. Choose **WinUSB** as the target driver and click **Replace Driver**.
6. Refresh Chrome and click **Connect**.

