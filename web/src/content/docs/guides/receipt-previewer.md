---
title: Virtual Receipt Previewer
description: Render pixel-perfect vector SVG and responsive HTML previews of thermal receipts on web dashboards and POS screens without physical hardware.
---

Testing thermal receipt layouts on physical hardware can be slow and wastes paper. Moreover, modern POS systems, order management dashboards, and e-commerce checkout flows frequently need to display a **realistic on-screen preview** of what the printed slip will look like.

`papermint` provides a built-in virtual preview engine capable of transforming any `Receipt` builder into:
1. **Vector SVG (`.render_svg()`)**: Scalable, resolution-independent vector graphic with realistic thermal paper drop shadows, monospace character grids, vector barcodes, QR blocks, and a serrated paper cut edge.
2. **HTML Snippet (`.render_html()`)**: Self-contained component snippet styled with inline CSS, ready for injection into React, Next.js, Vue, or customer-facing web screens.

---

## Why Virtual Previews?

- **Zero Paper Waste During Development**: Iterate on font sizes, table column alignments, dividers, and barcodes directly in your browser.
- **Order History & Back-Office Dashboards**: Show store managers and cashiers an exact visual replica of the printed ticket in web portals.
- **Customer Checkout Screens**: Display realistic digital receipts on secondary customer-facing POS tablets.
- **Headless PDF / Image Generation**: Convert SVG to PNG or PDF using standard tools for email attachments.

---

## TypeScript / Node.js Usage

```typescript
import { Receipt } from 'papermint';

const receipt = new Receipt('80mm')
  .init()
  .center()
  .bold(true)
  .doubleSize(true)
  .textLn('MINT BISTRO')
  .doubleSize(false)
  .bold(false)
  .textLn('123 Green Avenue, Suite 4B')
  .dividerDouble()
  .tableHeader(
    ['QTY', 'ITEM', 'PRICE', 'TOTAL'],
    [
      { widthFixed: 4, align: 'left' },
      { widthFraction: 0.5, align: 'left' },
      { widthFraction: 0.22, align: 'right' },
      { widthFraction: 0.24, align: 'right' },
    ]
  )
  .row(['1x', 'Truffle Wagyu Burger', '$14.50', '$14.50'])
  .row(['2x', 'Espresso Tonic', '$4.00', '$8.00'])
  .dividerDashed()
  .twoColumn('BALANCE DUE:', '$22.50')
  .feed(1)
  .qr('https://mintbistro.com/order/9981')
  .barcode128('TX-9981')
  .cutFull();

// 1. Generate SVG vector graphic
const svgContent = receipt.renderSvg();

// 2. Generate responsive HTML snippet
const htmlContent = receipt.renderHtml();
```

---

## React / Next.js Component Example

You can render the preview directly into any React or Next.js page:

```tsx
// components/ReceiptPreviewModal.tsx
import React from 'react';

interface ReceiptPreviewProps {
  receiptHtml: string;
  onClose: () => void;
}

export function ReceiptPreviewModal({ receiptHtml, onClose }: ReceiptPreviewProps) {
  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center p-4 z-50">
      <div className="bg-slate-100 rounded-lg p-6 max-w-lg w-full max-h-[90vh] overflow-y-auto">
        <div className="flex justify-between items-center mb-4">
          <h3 className="font-bold text-lg text-slate-800">Thermal Receipt Preview</h3>
          <button
            onClick={onClose}
            className="text-slate-500 hover:text-slate-800 text-sm font-semibold"
          >
            ✕ Close
          </button>
        </div>

        {/* Inject the pre-rendered, sandboxed receipt HTML */}
        <div
          className="receipt-viewport flex justify-center"
          dangerouslySetInnerHTML={{ __html: receiptHtml }}
        />
      </div>
    </div>
  );
}
```

---

## Rust Usage

In Rust desktop applications (e.g., Slint, Tauri, Iced) or backend microservices:

```rust
use papermint::{Receipt, PaperWidth};

fn main() {
    let receipt = Receipt::new(PaperWidth::Mm80)
        .center()
        .bold(true)
        .text_ln("MINT CAFE")
        .bold(false)
        .two_column("Iced Matcha Latte", "$5.50")
        .divider('-')
        .two_column("TOTAL", "$5.50")
        .cut_full();

    // Generate standalone SVG
    let svg = receipt.render_svg();
    std::fs::write("receipt_preview.svg", svg).unwrap();

    // Generate HTML snippet
    let html = receipt.render_html();
    std::fs::write("receipt_preview.html", html).unwrap();
}
```

---

## Visual Features Supported

| Feature | SVG Vector Preview | HTML Preview |
| :--- | :--- | :--- |
| **Monospace Typography** | Accurate character columns (ui-monospace) | Native system monospace fonts |
| **Text Alignment** | Left, Center, Right | `text-align: left \| center \| right` |
| **Bold & Underline** | `<tspan font-weight="bold">` | `<b>` & `text-decoration: underline` |
| **Inverted Printing** | High-contrast black box with white text | `background: #000; color: #fff` |
| **Double Height & Width** | Scaled vector fonts | Proportional font scaling |
| **Paper Widths** | 80mm (576 dots), 58mm (384 dots), Custom | Max-width matching physical dimensions |
| **Dividers & Rules** | Dashed, dotted, double, pattern rules | Accurate monospace repeating dividers |
| **QR Codes** | Sharp vector matrix with 3 finder patterns | Embedded crisp vector SVG |
| **1D Barcodes** | Vertical bar series with centered text label | Embedded crisp vector SVG |
| **Paper Tear / Cut Edge** | Jagged serrated bottom edge + cut lines | Dashed cut marker with ✂ indicator |

