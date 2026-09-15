/**
 * 100% PURE ROYAL ARABIC THERMAL RECEIPT
 * Designed for High-End Royal & VIP Hospitality (قصر الضيافة الملكية)
 *
 * Highlights:
 * - 100% Pure Arabic phrasing, dignified royal tone
 * - Zero English words or letters anywhere
 * - Authentic Emirati / Gulf royal hospitality menu (العود الملكي، القهوة العربية الذهبية، الحلوى الملكية)
 * - Pure Arabic currency: درهم إماراتي
 * - 6-line feed margin for complete cutter blade safety
 *
 * Usage:
 *   node examples/print_arabic_bill.mjs
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const nodePkgPath = path.resolve(__dirname, '../crates/papermint-node/index.js');
const { Receipt } = await import(nodePkgPath);

const receipt = new Receipt('80mm');

receipt.init()
  // Activate WPC1256 (Table 33 on PosBox PB800 / Rongta / Xprinter)
  .codePage(33)

  // ---------------------------------------------------------
  // 1. الترويسة الملكية الفاخرة (Royal Dignitary Header)
  // ---------------------------------------------------------
  .center()
  .doubleSize(true)
  .bold(true)
  .textLn('مجلس الضيافة الملكية')
  .doubleSize(false)
  .feed(1)
  .textLn('قصر الضيافة والمؤتمرات - دبي')
  .textLn('الإمارات العربية المتحدة')
  .feed(1)
  .bold(true)
  .textLn('فاتورة ضيافة رسمية معتمدة')
  .bold(false)
  .textLn('الرقم الضريبي المعتمد: ١٠٠٢٣٤٥٦٧٨٠٠٠٠٣')
  .feed(1)

  // ---------------------------------------------------------
  // 2. بيانات الفاتورة والجلسة (Session & Table Details)
  // ---------------------------------------------------------
  .dividerDouble()
  .left()
  .twoColumn('رقم الإيصال: ٩٠٠١', 'التاريخ: ١٥ سبتمبر ٢٠٢٦')
  .twoColumn('المجلس: الجناح الملكي (١)', 'الوقت: ١١:٠٠ صباحاً')
  .twoColumn('المضيف: سمو الشيخ والمرافقين', 'القاعة: القاعة الكبرى')
  .twoColumn('المسؤول: أحمد المنصوري', 'الجهاز: المحطة الملكية')
  .dividerDashed()

  // ---------------------------------------------------------
  // 3. جدول بنود الضيافة الفاخرة (Royal Menu Items)
  // ---------------------------------------------------------
  .tableHeader(['العدد', 'بيان الضيافة والخدمة الملكية', 'المبلغ'], [
    { widthFixed: 6, align: 'left' },
    { widthFraction: 1.0, align: 'left' },
    { widthFixed: 14, align: 'right' },
  ])
  .row(['١', 'دلة القهوة العربية الذهبية بالزعفران والهيل الملكي', '١٢٠٫٠٠ درهم'])
  .row(['١', 'باقة تمور الخلاص والسكري الفاخرة مع القشطة الطازجة', '٩٥٫٠٠ درهم'])
  .row(['٢', 'شاي كرك إماراتي فاخر بالزعفران الكشميري الأصيل', '٥٠٫٠٠ درهم'])
  .row(['١', 'حلوى أم علي الملكية بالفستق الحلبي والمكسرات المحمصة', '٨٥٫٠٠ درهم'])
  .row(['١', 'تشكيلة المعجنات الملكية الفاخرة بالزعتر والجبن البلدي', '٧٠٫٠٠ درهم'])
  .row(['١', 'بخور العود الملكي المعطر وقناديل الضيافة التراثية', '١٥٠٫٠٠ درهم'])
  .clearColumns()
  .divider()

  // ---------------------------------------------------------
  // 4. الحساب المالي والضريبة (Financial Summary in Dirhams)
  // ---------------------------------------------------------
  .right()
  .bold(true)
  .twoColumn('المجموع غير شامل الضريبة:', '٥٧٠٫٠٠ درهم')
  .twoColumn('ضريبة القيمة المضافة (٥٪):', '٢٨٫٥٠ درهم')
  .dividerDouble()

  // ---------------------------------------------------------
  // 5. الإجمالي المستحق بالدرهم الإماراتي (Grand Total Highlight)
  // ---------------------------------------------------------
  .doubleSize(true)
  .bold(true)
  .twoColumn('الإجمالي المستحق:', '٥٩٨٫٥٠ درهم')
  .doubleSize(false)
  .bold(false)
  .dividerDouble()

  // ---------------------------------------------------------
  // 6. رمز الاستجابة السريعة المعتمد (Certified FTA / ZATCA QR)
  // ---------------------------------------------------------
  .center()
  .feed(1)
  .textLn('امسح للتحقق من الفاتورة الضريبية المعتمدة')
  .textLn('الهيئة الاتحادية للضرائب - دولة الإمارات')
  .zatcaQr({
    sellerName: 'مجلس الضيافة الملكية ذ م م',
    vatNumber: '100234567800003',
    timestamp: new Date().toISOString(),
    totalAmount: '598.50',
    vatAmount: '28.50',
  })
  .feed(1)

  // ---------------------------------------------------------
  // 7. تحية التقدير والتوديع الملكي (Royal Farewell & Cutter Margin)
  // ---------------------------------------------------------
  .textLn('دمتم في حفظ الله ورعايته')
  .textLn('نتشرف دائماً بحضوركم وخدمتكم الكريمة')
  .textLn('أهلاً وسهلاً بكم في دار زايد')
  // 6 blank lines feed so the mechanical blade NEVER slices through the text!
  .feed(6)
  .cutFull();

// Compile ESC/POS binary
const bytes = receipt.encode('escpos');
const outputPath = path.resolve(__dirname, '../receipt.bin');
fs.writeFileSync(outputPath, bytes);

console.log('====================================================');
console.log('👑 ROYAL ARABIC RECEIPT (مجلس الضيافة الملكية) COMPILED!');
console.log(`📁 File: ${outputPath} (${bytes.length} bytes)`);
console.log('====================================================');
console.log('Ready to print on your POS-80 via temp/print_receipt.bat');
console.log('====================================================');
