/**
 * 🌿 PAPERMINT SHOWCASE RECEIPT
 * Full Bilingual (English & Arabic) + N-Column Table Engine + QR + Barcode
 *
 * Designed to showcase all v0.3.0 layout features on an 80mm thermal receipt printer.
 *
 * Usage:
 *   node examples/print_showcase_bill.mjs
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const nodePkgPath = path.resolve(__dirname, '../crates/papermint-node/index.js');
const { Receipt } = await import(nodePkgPath);

const receipt = new Receipt('80mm');

receipt
  .init()
  // Select Table 33 (WPC1256 Arabic for POS-80 / Rongta / Xprinter / PosBox)
  .codePage(33)

  // 1. BRAND HEADER
  .center()
  .bold(true)
  .doubleSize(true)
  .textLn('PAPERMINT BISTRO')
  .doubleSize(false)
  .textLn('مقهى وبيسترو بيبرمنت')
  .bold(false)
  .feed(1)
  .textLn('Downtown Boulevard - Floor 2')
  .textLn('VAT / TRN: 300459281700003')
  .feed(1)

  // 2. ORDER METADATA (Two-Column)
  .dividerDouble()
  .left()
  .twoColumn('Order: #1042', 'Table: 08 (Dine-In)')
  .twoColumn('Date: 16 Sep 2026', 'Time: 01:30 PM')
  .twoColumn('Server: Sarah M.', 'Station: POS-01')
  .dividerDashed()

  // 3. ADVANCED N-COLUMN TABLE WITH WORD-WRAPPING
  .tableHeader(
    ['Qty', 'Item Description', 'Price', 'Total'],
    [
      { widthFixed: 4, align: 'left' },
      { widthFraction: 0.52, align: 'left' },
      { widthFixed: 9, align: 'right' },
      { widthFixed: 9, align: 'right' },
    ]
  )
  .row(['2x', 'Double Truffle Angus Smash Burger with Onion Jam', '$12.00', '$24.00'])
  .row(['1x', 'Artisanal Sparkling Mint Cold Brew (Cold Foam)', '$5.50', '$5.50'])
  .row(['1x', 'Single Origin Ethiopian Pour-Over Coffee (V60)', '$8.00', '$8.00'])
  .row(['2x', 'Warm Cinnamon Brioche French Toast & Berries', '$6.50', '$13.00'])
  .clearColumns()
  .divider()

  // 4. FINANCIAL SUMMARY
  .right()
  .twoColumn('Subtotal (Net):', '$50.50')
  .twoColumn('Tax / VAT (5%):', '$2.53')
  .dividerDouble()
  .bold(true)
  .doubleSize(true)
  .twoColumn('TOTAL DUE:', '$53.03')
  .doubleSize(false)
  .bold(false)
  .dividerDouble()

  // 5. BARCODE (Code 128)
  .center()
  .feed(1)
  .textLn('ORDER TRACKING BARCODE')
  .barcode128('PM-2026-1042')
  .feed(1)

  // 6. 2D SCANNABLE QR CODE
  .textLn('Scan to View Digital e-Receipt')
  .qr('https://papermint.dev/receipt/1042')
  .feed(1)

  // 7. ROYAL FAREWELL & BLADE MARGIN
  .bold(true)
  .textLn('Thank You For Visiting!')
  .textLn('نتشرف دائماً بزيارتكم الكريمة')
  .bold(false)
  // 6 feed lines ensure the mechanical blade never cuts through the QR or text!
  .feed(6)
  .cutFull();

// Compile binary wire bytes for ESC/POS
const bytes = receipt.encode('escpos');
const outputPath = path.resolve(__dirname, '../receipt.bin');
fs.writeFileSync(outputPath, bytes);

// Generate virtual SVG preview for screen viewing
const svg = receipt.renderSvg();
const svgPath = path.resolve(__dirname, '../receipt_preview.svg');
fs.writeFileSync(svgPath, svg);

console.log('====================================================');
console.log('🌿 PAPERMINT SHOWCASE RECEIPT COMPILED SUCCESSFULLY!');
console.log(`📁 Binary Spool File: ${outputPath} (${bytes.length} bytes)`);
console.log(`🎨 Virtual SVG Preview: ${svgPath}`);
console.log('====================================================');
console.log('Ready to print on your POS-80 via temp/print_receipt.bat');
console.log('====================================================');

