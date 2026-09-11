/**
 * 80mm Arabic POS Thermal Receipt Example (Node.js / ESC-POS)
 *
 * Demonstrates:
 * - Bilingual Arabic & English receipt formatting on 80mm thermal paper
 * - Windows-1256 / WPC1256 Arabic character table activation (CodePage 33 on PosBox PB800)
 * - Middle East ZATCA (Saudi Arabia) & FTA (UAE) compliant E-Invoicing QR code with Arabic seller name
 * - Multiline table layout with SAR currency
 * - Codepage & RTL direction comparison diagnostic strip for physical hardware inspection
 *
 * Run from repository root:
 *   node examples/print_arabic_bill.mjs
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const nodePkgPath = path.resolve(__dirname, '../crates/papermint-node/index.js');
const { Receipt } = await import(nodePkgPath);

function reverseText(str) {
  return str.split('').reverse().join('');
}

// 1. Initialize 80mm receipt builder
const receipt = new Receipt('80mm');

receipt.init()
  // Activate WPC1256 (Table 33 on PosBox PB800 / Chinese OEM thermal printers)
  .codePage(33)

  // ---------------------------------------------------------
  // Header: Bilingual Restaurant Branding
  // ---------------------------------------------------------
  .center()
  .doubleSize(true)
  .bold(true)
  .textLn('PAPERMINT BISTRO')
  .doubleSize(false)
  .textLn('مقهى ومخبز بيبرمينت')
  .bold(false)
  .feed(1)
  .textLn('Artisan Arabic Coffee & Gourmet Bakery')
  .textLn('فاتورة ضريبية مبسطة - SIMPLIFIED TAX INVOICE')
  .textLn('الرقم الضريبي / VAT TRN: 310123456700003')
  .textLn('Tel: +966 11 456 7890 - Riyadh, KSA')
  .feed(1)

  // ---------------------------------------------------------
  // Metadata Grid
  // ---------------------------------------------------------
  .dividerDouble()
  .left()
  .twoColumn('Invoice: #PM-8924', 'Date: 11-Sep-2026')
  .twoColumn('Table: T-04 (العائلات)', 'Time: 05:15 PM')
  .twoColumn('Cashier: أحمد (Ahmed)', 'Terminal: POS-01')
  .dividerDashed()

  // ---------------------------------------------------------
  // Multiline Items Table (Bilingual Items & SAR Currency)
  // ---------------------------------------------------------
  .tableHeader(['QTY', 'DESCRIPTION / الصنف', 'AMOUNT'], [
    { widthFixed: 4, align: 'left' },
    { widthFraction: 1.0, align: 'left' },
    { widthFixed: 12, align: 'right' },
  ])
  .row(['1x', 'Premium Arabic Coffee with Dates\nقهوة عربية فاخرة مع التمر', 'SAR 25.00'])
  .row(['2x', 'Karak Tea with Saffron & Cardamom\nشاي كرك بالزعفران والهيل', 'SAR 18.00'])
  .row(['1x', 'Umm Ali with Nuts & Fresh Cream\nحلوى أم علي بالمكسرات والقشطة', 'SAR 32.00'])
  .row(['1x', 'Fresh Pistachio Croissant\nكرواسون الفستق الطازج', 'SAR 20.00'])
  .clearColumns()
  .divider()

  // ---------------------------------------------------------
  // Financial Summary
  // ---------------------------------------------------------
  .right()
  .bold(true)
  .twoColumn('SUBTOTAL / المجموع الفرعي:', 'SAR  95.00')
  .twoColumn('VAT (15%) / ضريبة القيمة المضافة:', 'SAR  14.25')
  .dividerDouble()

  // ---------------------------------------------------------
  // Grand Total Highlight
  // ---------------------------------------------------------
  .doubleSize(true)
  .bold(true)
  .twoColumn('TOTAL / الإجمالي:', 'SAR 109.25')
  .doubleSize(false)
  .bold(false)
  .dividerDouble()

  // ---------------------------------------------------------
  // Saudi ZATCA E-Invoicing TLV QR Code
  // (Tag 1 seller name in UTF-8 Arabic decodes natively on smartphones!)
  // ---------------------------------------------------------
  .center()
  .feed(1)
  .textLn('Scan for ZATCA E-Invoice Verification:')
  .textLn('امسح للتحقق من الفاتورة الضريبية')
  .zatcaQr({
    sellerName: 'مؤسسة بيبرمينت لتقديم الوجبات',
    vatNumber: '310123456700003',
    timestamp: new Date().toISOString(),
    totalAmount: '109.25',
    vatAmount: '14.25',
  })
  .feed(1)

  // ---------------------------------------------------------
  // Hardware Codepage & RTL Diagnostic Comparison Strip
  // (Allows visual verification of your PB800 printer font ROM)
  // ---------------------------------------------------------
  .left()
  .dividerPattern('=-')
  .center()
  .bold(true)
  .textLn('--- HARDWARE ARABIC CODEPAGE TEST ---')
  .bold(false)
  .left()

  // Test 1: CodePage 33 (WPC1256) - Logical
  .codePage(33)
  .textLn('1. CP33 (WPC1256 - Logical):')
  .textLn('   فاتورة ضريبية - قهوة عربية')

  // Test 2: CodePage 33 (WPC1256) - Reversed (RTL)
  .codePage(33)
  .textLn('2. CP33 (WPC1256 - RTL Reversed):')
  .textLn('   ' + reverseText('فاتورة ضريبية - قهوة عربية'))

  // Test 3: CodePage 22 (OEM Arabic) - Logical
  .codePage(22)
  .textLn('3. CP22 (Arabic OEM - Logical):')
  .textLn('   فاتورة ضريبية - قهوة عربية')

  // Test 4: CodePage 22 (OEM Arabic) - Reversed (RTL)
  .codePage(22)
  .textLn('4. CP22 (Arabic OEM - RTL Reversed):')
  .textLn('   ' + reverseText('فاتورة ضريبية - قهوة عربية'))

  // Test 5: CodePage 63 (PC864 Arabic) - Logical
  .codePage(63)
  .textLn('5. CP63 (PC864 - Logical):')
  .textLn('   فاتورة ضريبية - قهوة عربية')

  // Test 6: CodePage 63 (PC864 Arabic) - Reversed (RTL)
  .codePage(63)
  .textLn('6. CP63 (PC864 - RTL Reversed):')
  .textLn('   ' + reverseText('فاتورة ضريبية - قهوة عربية'))

  .dividerPattern('=-')

  // Reset back to standard CodePage 0 (PC437)
  .codePage(0)
  .center()
  .feed(1)
  .textLn('Thank you for dining with us!')
  .textLn('شكراً لزيارتكم! نتشرف بحضوركم دائماً')
  .textLn('papermint.zerathlabs.com')
  .feed(4)
  .cutFull();

// Compile ESC/POS binary
const bytes = receipt.encode('escpos');
const outputPath = path.resolve(__dirname, '../receipt.bin');
fs.writeFileSync(outputPath, bytes);

console.log('====================================================');
console.log('✅ Arabic Receipt compiled successfully!');
console.log(`📁 File: ${outputPath} (${bytes.length} bytes)`);
console.log('====================================================');
console.log('To print on your physical PosBox PB800:');
console.log('  Windows PowerShell:');
console.log('    .\\temp\\send_receipt.ps1');
console.log('====================================================');

