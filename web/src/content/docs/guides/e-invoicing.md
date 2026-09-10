---
title: E-Invoicing Compliance (ZATCA & UAE FTA)
description: Generate tax-compliant TLV Base64 QR codes for electronic invoicing in Saudi Arabia (ZATCA / FATOORA) and the United Arab Emirates (FTA).
---

The tax authorities across the GCC—notably the **Saudi Zakat, Tax and Customs Authority (ZATCA / FATOORA)** and the **United Arab Emirates Federal Tax Authority (FTA)**—mandate that all point-of-sale simplified tax invoices include a specific **2D QR Code**.

Unlike typical QR codes that simply contain a website URL, fiscal QR codes in the Middle East encode a binary **TLV (Tag-Length-Value)** data structure serialized into a standard **RFC 4648 Base64** string.

`papermint` provides built-in, dependency-free TLV packing and Base64 generation in both **Rust** and **Node.js/TypeScript**.

---

## TLV Specification & Required Fields

Each fiscal field is encoded as:
- **Tag**: 1 byte (0x01 to 0x09)
- **Length**: 1 byte (length of the value in bytes)
- **Value**: UTF-8 string or binary payload

### Phase 1 (Generation Phase) Mandatory Fields

| Tag | Name | Format / Example | Description |
| :---: | :--- | :--- | :--- |
| **0x01** | **Seller's Name** | `Bob's Fashions` | Legal or commercial trading name of the seller. |
| **0x02** | **VAT Registration No (TRN)** | `310122393500003` | 15-digit VAT number issued by ZATCA or FTA. |
| **0x03** | **Invoice Timestamp** | `2026-09-10T14:30:00Z` | Issue date and time in ISO 8601 UTC format. |
| **0x04** | **Invoice Total** | `1000.00` | Total bill amount including VAT in decimal format. |
| **0x05** | **Total VAT Amount** | `150.00` | Total tax amount (e.g., 15% in KSA, 5% in UAE). |

### Phase 2 (Integration Phase) Cryptographic Fields

| Tag | Name | Type | Description |
| :---: | :--- | :--- | :--- |
| **0x06** | **Invoice Hash** | SHA-256 (32 bytes) | Cryptographic hash of the simplified tax invoice. |
| **0x07** | **Digital Signature** | ECDSA (64–72 bytes) | Cryptographic signature of the invoice hash. |
| **0x08** | **Public Key** | ECDSA (64–88 bytes) | Public key used to generate the signature. |
| **0x09** | **Cryptographic Stamp** | Binary | Optional ZATCA cryptographic stamp. |

---

## TypeScript / Node.js Usage

In your Node.js backend (e.g., Express, Hono, NestJS, Next.js):

```typescript
import { Receipt, zatcaQrBase64 } from 'papermint';

// 1. Define compliant invoice metadata
const invoice = {
  sellerName: 'Al-Madina Gourmet Bistro',
  vatNumber: '310122393500003', // 15-digit Tax ID
  timestamp: new Date().toISOString(),
  totalAmount: '115.00',
  vatAmount: '15.00', // 15% VAT for KSA
};

// 2. Build and print the thermal receipt
const receipt = new Receipt('80mm')
  .init()
  .center()
  .bold(true)
  .textLn('AL-MADINA GOURMET BISTRO')
  .bold(false)
  .textLn('Tax Invoice #INV-2026-8801')
  .textLn(`TRN: ${invoice.vatNumber}`)
  .divider('-')
  .twoColumn('Chicken Mandi (Full)', 'SAR 86.96')
  .twoColumn('Fresh Mint Lemonade', 'SAR 13.04')
  .divider('-')
  .twoColumn('Subtotal (Excl. VAT)', 'SAR 100.00')
  .twoColumn('VAT (15%)', 'SAR 15.00')
  .dividerDouble()
  .bold(true)
  .twoColumn('TOTAL DUE', 'SAR 115.00')
  .bold(false)
  .feed(1)
  // Appends the compliant ZATCA TLV Base64 QR code
  .zatcaQr(invoice)
  .feed(2)
  .cutFull();

// 3. Output wire bytes (ESC/POS or StarPRNT)
const escposBytes = receipt.encode('escpos');
```

### Standalone Base64 Helper

If you only need the Base64 QR payload string without building a receipt (for example, to store in your database or display on a website):

```typescript
import { zatcaQrBase64 } from 'papermint';

const payload = zatcaQrBase64(
  'Dubai Marina Cafe LLC',
  '100234567800003',
  '2026-09-10T14:30:00Z',
  '85.00',
  '4.25' // 5% VAT in UAE
);

console.log(payload);
// Output: AQVE... (TLV Base64 string)
```

---

## Rust Usage

In high-performance Rust POS servers or edge microservices:

```rust
use papermint::tax::ZatcaInvoice;
use papermint::{PaperWidth, Receipt};

fn generate_tax_receipt() -> Receipt {
    // 1. Configure Phase 1 mandatory tax fields
    let invoice = ZatcaInvoice::new(
        "Bob's Fashions",
        "310122393500003",
        "2026-09-10T15:30:00Z",
        "1000.00",
        "150.00",
    );

    // 2. Compose receipt
    Receipt::new(PaperWidth::Mm80)
        .init()
        .center()
        .bold(true)
        .text_ln("BOB'S FASHIONS")
        .bold(false)
        .two_column("Subtotal", "SAR 850.00")
        .two_column("VAT (15%)", "SAR 150.00")
        .divider('=')
        .two_column("TOTAL", "SAR 1000.00")
        .feed(1)
        // Fluent ZATCA QR builder
        .zatca_qr(&invoice)
        .feed(2)
        .cut_full()
}
```

### Phase 2 Cryptographic Fields (Rust)

For Phase 2 integration, use the fluent builder methods to attach cryptographic elements:

```rust
let sha256_hash = vec![/* 32 bytes */];
let ecdsa_signature = vec![/* 64-72 bytes */];
let public_key = vec![/* 64-88 bytes */];

let phase2_invoice = ZatcaInvoice::new(
    "Saudi Tech Enterprises",
    "310000000000003",
    "2026-09-10T12:00:00Z",
    "500.00",
    "75.00",
)
.with_invoice_hash(sha256_hash)
.with_signature(ecdsa_signature)
.with_public_key(public_key);

let qr_base64 = phase2_invoice.to_qr_base64();
```

---

## Verification & Testing with Official Apps

When printed on a physical thermal receipt or previewed on screen, scan the QR code with:
- **ZATCA Official App** (available on iOS App Store and Google Play)
- **VAT QR Scanner UAE** (FTA compliance scanner)
- Any standard TLV Base64 barcode scanner

The scanner will parse the Base64 stream, decode the TLV tags, and display:
- **Seller Name**: Authenticated
- **VAT Number**: Verified
- **Date & Time**: Verified
- **Total Amount**: Verified
- **Tax Amount**: Verified

