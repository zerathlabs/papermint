import assert from 'node:assert';
import { Receipt, Printer, availablePorts, listPrinters, zatcaQrBase64 } from './index.js';



console.log('🧪 Testing papermint Node.js/TypeScript Bindings...\n');

// 1. Test Receipt Builder & Table Engine
console.log('1. Testing fluent Receipt builder...');
const receipt = new Receipt('80mm')
  .init()
  .center()
  .bold(true)
  .doubleSize(true)
  .textLn('MINT BISTRO')
  .doubleSize(false)
  .bold(false)
  .textLn('123 Main Street')
  .dividerDouble()
  .tableHeader(
    ['QTY', 'DESCRIPTION', 'PRICE', 'TOTAL'],
    [
      { widthFixed: 4, align: 'left' },
      { widthFraction: 0.50, align: 'left' },
      { widthFraction: 0.22, align: 'right' },
      { widthFraction: 0.24, align: 'right' },
    ]
  )
  .row(['2x', 'Truffle Wagyu Burger with Caramelized Onions', '$14.50', '$29.00'])
  .row(['1x', 'Wood-Fired Margherita Pizza', '$18.00', '$18.00'])
  .dividerDashed()
  .twoColumn('BALANCE DUE:', '$47.00')
  .dividerDotted()
  .dividerPattern('=-')
  .qr('https://pay.mintbistro.com/bill/1042')
  .barcode128('TX-1042')
  .cutFull();

assert(receipt instanceof Receipt, 'receipt should be an instance of Receipt');
console.log('   ✅ Receipt created successfully with fluent chaining.');

// 2. Test Direct Buffer Encoding (ESC/POS and Star)
console.log('\n2. Testing binary wire encoding to Node Buffers...');
const escposBytes = receipt.encode('escpos');
assert(Buffer.isBuffer(escposBytes), 'encoded ESC/POS output must be a Buffer');
assert(escposBytes.length > 0, 'ESC/POS buffer should not be empty');
console.log(`   ✅ Encoded ${escposBytes.length} ESC/POS wire bytes.`);

const starBytes = receipt.encode('star');
assert(Buffer.isBuffer(starBytes), 'encoded StarPRNT output must be a Buffer');
assert(starBytes.length > 0, 'StarPRNT buffer should not be empty');
console.log(`   ✅ Encoded ${starBytes.length} StarPRNT wire bytes.`);

// 3. Test In-Memory Mock Printer
console.log('\n3. Testing Printer.mock() transmission...');
const mockPrinter = Printer.mock('escpos');
await mockPrinter.print(receipt);

const recordedBytes = await mockPrinter.getRecordedBytes();
assert(Buffer.isBuffer(recordedBytes), 'recordedBytes must be a Buffer');
assert.strictEqual(recordedBytes.length, escposBytes.length, 'recorded bytes must match encoded receipt');
console.log(`   ✅ Mock printer recorded ${recordedBytes.length} bytes correctly.`);

// 4. Test Device Discovery Helpers
console.log('\n4. Testing hardware device discovery functions...');
const ports = availablePorts();
assert(Array.isArray(ports), 'availablePorts must return an array of strings');
console.log(`   ✅ availablePorts returned ${ports.length} port(s): ${JSON.stringify(ports)}`);

const usbPrinters = listPrinters();
assert(Array.isArray(usbPrinters), 'listPrinters must return an array of devices');
console.log(`   ✅ listPrinters returned ${usbPrinters.length} printer(s).`);

// 5. Test ZATCA & UAE E-Invoicing Engine
console.log('\n5. Testing ZATCA & UAE E-Invoicing QR engine...');
const zatcaInvoice = {
  sellerName: "Bob's Fashions",
  vatNumber: '310122393500003',
  timestamp: '2022-04-25T15:30:00Z',
  totalAmount: '1000.00',
  vatAmount: '150.00',
};

const base64Qr = zatcaQrBase64(
  zatcaInvoice.sellerName,
  zatcaInvoice.vatNumber,
  zatcaInvoice.timestamp,
  zatcaInvoice.totalAmount,
  zatcaInvoice.vatAmount,
);
assert(typeof base64Qr === 'string', 'zatcaQrBase64 must return a string');
assert(base64Qr.length > 0, 'Base64 QR payload cannot be empty');
console.log(`   ✅ zatcaQrBase64 payload generated (${base64Qr.length} chars).`);

const taxReceipt = new Receipt('80mm')
  .center()
  .bold(true)
  .textLn("BOB'S FASHIONS")
  .bold(false)
  .twoColumn('Subtotal', 'SAR 850.00')
  .twoColumn('VAT (15%)', 'SAR 150.00')
  .twoColumn('TOTAL', 'SAR 1000.00')
  .feed(1)
  .zatcaQr(zatcaInvoice)
  .cutFull();

const taxEscpos = taxReceipt.encode('escpos');
assert(Buffer.isBuffer(taxEscpos) && taxEscpos.length > 0);
console.log(`   ✅ Receipt.zatcaQr() integrated and encoded (${taxEscpos.length} bytes).`);

// 6. Test Virtual Receipt Previewer (SVG & HTML)
console.log('\n6. Testing Virtual Receipt Previewer (SVG & HTML)...');
const svg = receipt.renderSvg();
assert(typeof svg === 'string', 'renderSvg must return string');
assert(svg.startsWith('<svg'), 'SVG must start with <svg');
assert(svg.includes('MINT BISTRO'), 'SVG must contain receipt text');
assert(svg.includes('✂ CUT'), 'SVG must include cut indicator');
console.log(`   ✅ renderSvg() generated valid SVG (${svg.length} bytes).`);

const html = receipt.renderHtml();
assert(typeof html === 'string', 'renderHtml must return string');
assert(html.includes('papermint-receipt'), 'HTML must include receipt container class');
assert(html.includes('MINT BISTRO'), 'HTML must contain receipt text');
console.log(`   ✅ renderHtml() generated valid HTML (${html.length} bytes).`);

console.log('\n🎉 ALL Node.js N-API binding tests passed successfully!');


