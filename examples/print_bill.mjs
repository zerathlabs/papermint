/**
 * 80mm POS Thermal Receipt Example (Node.js / ESC-POS)
 *
 * Demonstrates:
 * - 80mm monospace grid formatting (Font A, 48 columns)
 * - Synchronized multiline table row wrapping
 * - Middle East ZATCA & UAE compliant E-Invoicing TLV QR code
 * - Auto-cutter and acoustic buzzer
 *
 * Run from repository root:
 *   node examples/print_bill.mjs
 */

import fs from 'fs';
import { Receipt } from './crates/papermint-node/index.js';

// Create 80mm receipt builder
const receipt = new Receipt('80mm');

receipt.init()
  // 1. Store Header
  .center()
  .doubleHeight(true)
  .bold(true)
  .textLn('PAPERMINT BISTRO')
  .doubleHeight(false)
  .bold(false)
  .textLn('Artisan Coffee & Gourmet Kitchen')
  .textLn('GSTIN: 07AAAAA0000A1Z5')
  .textLn('Tel: +91 94296 92464')
  .feed(1)

  // 2. Metadata Grid
  .dividerDouble()
  .left()
  .twoColumn('Invoice No: #PM-8924', 'Date: 11-Sep-2026')
  .twoColumn('Table: T-04 (Indoor)', 'Time: 03:25 PM')
  .twoColumn('Server: Mohammed', 'Terminal: POS-01')
  .dividerDashed()

  // 3. Multiline Items Table
  .tableHeader(['QTY', 'DESCRIPTION', 'AMOUNT'], [
    { widthFixed: 4, align: 'left' },
    { widthFraction: 1.0, align: 'left' },
    { widthFixed: 12, align: 'right' },
  ])
  .row(['1x', 'Double Truffle Avocado Burger with Spicy Chipotle', 'Rs. 450.00'])
  .row(['2x', 'Artisan Cold Brew Coffee (Madagascar Vanilla)', 'Rs. 360.00'])
  .row(['1x', 'Parmesan Crinkle Truffle Fries', 'Rs. 180.00'])
  .clearColumns()
  .divider()

  // 4. Financial Summary
  .right()
  .bold(true)
  .twoColumn('SUBTOTAL:', 'Rs. 990.00')
  .twoColumn('CGST (2.5%):', 'Rs.  24.75')
  .twoColumn('SGST (2.5%):', 'Rs.  24.75')
  .dividerDouble()

  // 5. Grand Total Highlight
  .doubleSize(true)
  .bold(true)
  .twoColumn('GRAND TOTAL:', 'Rs. 1039.50')
  .doubleSize(false)
  .bold(false)
  .dividerDouble()

  // 6. Middle East ZATCA / UAE Compliant E-Invoicing QR Code
  .center()
  .feed(1)
  .textLn('Scan for E-Invoice Verification:')
  .zatcaQr({
    sellerName: 'Papermint Bistro Ltd',
    vatNumber: '310123456700003',
    timestamp: new Date().toISOString(),
    totalAmount: '1039.50',
    vatAmount: '49.50',
  })
  .feed(1)
  .textLn('Thank you for dining with us!')
  .textLn('Visit again: papermint.zerathlabs.com')
  .feed(4)
  .cutFull();

const bytes = receipt.encode('escpos');
fs.writeFileSync('receipt.bin', bytes);
console.log('Successfully compiled receipt.bin (' + bytes.length + ' bytes)');

