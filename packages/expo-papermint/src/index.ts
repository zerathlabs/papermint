// Reexport the native module. On web, it will be resolved to ExpoPapermintModule.web.ts
// and on native platforms to ExpoPapermintModule.ts
export { default as ExpoPapermintModule } from './ExpoPapermintModule';
export { default } from './ExpoPapermintModule';
import ExpoPapermintModule from './ExpoPapermintModule';
import type {
  CutMode,
  Dialect,
  PaperWidth,
  TableColumnOptions,
  TicketPayload,
  ZatcaQrParams,
} from './ExpoPapermint.types';

export * from './ExpoPapermint.types';
export { Label, compileLabel, renderLabelSvg } from './Label';
export {
  usePrinterGateway,
  printLocalOrder,
  printLocalLabel,
  PrinterGatewayClient,
  sendToPrinter,
} from './gateway';

/**
 * Compiles a receipt ticket specification into ESC/POS or StarPRNT binary wire bytes in ~10 microseconds.
 *
 * @param ticket The ticket definition object.
 * @param dialect Thermal printer protocol dialect ('escpos' or 'star'). Defaults to 'escpos'.
 * @returns Uint8Array of binary wire bytes ready to stream to a Bluetooth or Network printer.
 */
export function compileTicket(ticket: TicketPayload, dialect: Dialect = 'escpos'): Uint8Array {
  const jsonStr = JSON.stringify(ticket);
  return ExpoPapermintModule.compileTicket(jsonStr, dialect);
}

/**
 * Renders a receipt ticket specification into a virtual SVG vector graphic string.
 *
 * Simulates physical monospace thermal receipt paper with column alignment,
 * barcode vectors, and realistic paper edge rendering.
 *
 * @param ticket The ticket definition object.
 * @returns SVG string ready for rendering inside React Native SVG or WebViews.
 */
export function renderSvg(ticket: TicketPayload): string {
  const jsonStr = JSON.stringify(ticket);
  return ExpoPapermintModule.renderSvg(jsonStr);
}

/**
 * Renders a receipt ticket specification into a responsive HTML preview snippet.
 *
 * @param ticket The ticket definition object.
 * @returns HTML string with inline styling for embedding in checkout / cashier screens.
 */
export function renderHtml(ticket: TicketPayload): string {
  const jsonStr = JSON.stringify(ticket);
  return ExpoPapermintModule.renderHtml(jsonStr);
}

/**
 * Encodes Saudi Arabia ZATCA Phase 1 & 2 e-invoicing TLV (Tag-Length-Value) Base64 QR code.
 */
export function generateZatcaQr(params: ZatcaQrParams): string {
  const encodeTlv = (tag: number, val: string): Uint8Array => {
    const textEncoder = new TextEncoder();
    const strBytes = textEncoder.encode(val);
    const result = new Uint8Array(2 + strBytes.length);
    result[0] = tag;
    result[1] = strBytes.length;
    result.set(strBytes, 2);
    return result;
  };

  const parts = [
    encodeTlv(1, params.sellerName),
    encodeTlv(2, params.vatNumber),
    encodeTlv(3, params.timestamp),
    encodeTlv(4, params.totalAmount),
    encodeTlv(5, params.taxAmount),
  ];

  const totalLength = parts.reduce((acc, p) => acc + p.length, 0);
  const combined = new Uint8Array(totalLength);
  let offset = 0;
  for (const part of parts) {
    combined.set(part, offset);
    offset += part.length;
  }

  // Base64 encoding compatible with Hermes / React Native / Web
  if (typeof btoa === 'function') {
    let binary = '';
    for (let i = 0; i < combined.length; i++) {
      binary += String.fromCharCode(combined[i]);
    }
    return btoa(binary);
  } else {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
    let output = '';
    for (let i = 0; i < combined.length; i += 3) {
      const b1 = combined[i];
      const b2 = i + 1 < combined.length ? combined[i + 1] : 0;
      const b3 = i + 2 < combined.length ? combined[i + 2] : 0;
      output += chars[(b1 >> 2) & 0x3F];
      output += chars[((b1 & 0x03) << 4) | ((b2 >> 4) & 0x0F)];
      output += i + 1 < combined.length ? chars[((b2 & 0x0F) << 2) | ((b3 >> 6) & 0x03)] : '=';
      output += i + 2 < combined.length ? chars[b3 & 0x3F] : '=';
    }
    return output;
  }
}

/**
 * Fluent Receipt Builder for mobile POS apps.
 *
 * Supports both high-level structured tickets and granular low-level typography,
 * arbitrary multi-column tables, hardware code pages, images, and virtual SVG/HTML previewing.
 */
export class Receipt {
  private payload: TicketPayload;

  constructor(paperWidth: PaperWidth = '80mm') {
    this.payload = {
      paper_width: paperWidth,
      commands: [],
      metadata: [],
      items: [],
      totals: [],
    };
  }

  public static create(paperWidth: PaperWidth = '80mm'): Receipt {
    return new Receipt(paperWidth);
  }

  // --- Granular Typography & Text Commands ---

  public text(text: string): this {
    this.payload.commands?.push({ type: 'text', text });
    return this;
  }

  public textLn(text: string): this {
    this.payload.commands?.push({ type: 'text_ln', text });
    return this;
  }

  public left(): this {
    this.payload.commands?.push({ type: 'align', align: 'left' });
    return this;
  }

  public center(): this {
    this.payload.commands?.push({ type: 'align', align: 'center' });
    return this;
  }

  public right(): this {
    this.payload.commands?.push({ type: 'align', align: 'right' });
    return this;
  }

  public bold(enable: boolean = true): this {
    this.payload.commands?.push({ type: 'bold', enable });
    return this;
  }

  public underline(enable: boolean = true, mode: 'single' | 'double' = 'single'): this {
    this.payload.commands?.push({ type: 'underline', enable, mode });
    return this;
  }

  public invert(enable: boolean = true): this {
    this.payload.commands?.push({ type: 'invert', enable });
    return this;
  }

  public doubleWidth(enable: boolean = true): this {
    this.payload.commands?.push({ type: 'double_width', enable });
    return this;
  }

  public doubleHeight(enable: boolean = true): this {
    this.payload.commands?.push({ type: 'double_height', enable });
    return this;
  }

  public doubleSize(enable: boolean = true): this {
    this.payload.commands?.push({ type: 'double_size', enable });
    return this;
  }

  public feed(lines: number = 1): this {
    this.payload.commands?.push({ type: 'feed', lines });
    return this;
  }

  // --- Dividers & Rules ---

  public divider(style: string = '-'): this {
    this.payload.commands?.push({ type: 'divider', style, variant: 'single' });
    return this;
  }

  public dividerDouble(): this {
    this.payload.commands?.push({ type: 'divider', variant: 'double' });
    return this;
  }

  public dividerDotted(): this {
    this.payload.commands?.push({ type: 'divider', variant: 'dotted' });
    return this;
  }

  public dividerDashed(): this {
    this.payload.commands?.push({ type: 'divider', variant: 'dashed' });
    return this;
  }

  public dividerPattern(pattern: string): this {
    this.payload.commands?.push({ type: 'divider', pattern });
    return this;
  }

  // --- Multi-Column Tables ---

  public twoColumn(left: string, right: string): this {
    this.payload.commands?.push({ type: 'two_column', left, right });
    return this;
  }

  public threeColumn(left: string, center: string, right: string): this {
    this.payload.commands?.push({ type: 'three_column', left, center, right });
    return this;
  }

  public tableHeader(headers: string[], columns: TableColumnOptions[]): this {
    this.payload.commands?.push({ type: 'table_header', headers, columns });
    return this;
  }

  public setColumns(columns: TableColumnOptions[]): this {
    this.payload.commands?.push({ type: 'set_columns', columns });
    return this;
  }

  public row(cells: string[]): this {
    this.payload.commands?.push({ type: 'row', cells });
    return this;
  }

  public clearColumns(): this {
    this.payload.commands?.push({ type: 'clear_columns' });
    return this;
  }

  // --- Barcodes & QR ---

  public qr(content: string): this {
    this.payload.commands?.push({ type: 'qr', content });
    return this;
  }

  public zatcaQr(params: ZatcaQrParams): this {
    const b64 = generateZatcaQr(params);
    return this.qr(b64);
  }

  public barcode(code: string): this {
    this.payload.commands?.push({ type: 'barcode', content: code });
    return this;
  }

  public barcode128(code: string): this {
    return this.barcode(code);
  }

  // --- Hardware Commands & Code Pages ---

  public codePage(page: string | number): this {
    this.payload.code_page = page;
    this.payload.commands?.push({ type: 'code_page', page });
    return this;
  }

  public openDrawer(): this {
    this.payload.commands?.push({ type: 'open_drawer' });
    this.payload.open_drawer = true;
    return this;
  }

  public beep(count: number = 1, duration: number = 2): this {
    this.payload.commands?.push({ type: 'beep', count, duration });
    this.payload.beep = count;
    return this;
  }

  public cut(mode: CutMode = 'full'): this {
    this.payload.commands?.push({ type: 'cut', mode });
    this.payload.cut_mode = mode;
    return this;
  }

  public cutFull(): this {
    return this.cut('full');
  }

  public cutPartial(): this {
    return this.cut('partial');
  }

  public raw(bytes: Uint8Array | number[]): this {
    const arr = Array.from(bytes);
    this.payload.commands?.push({ type: 'raw', bytes: arr });
    return this;
  }

  public image(data: string | Uint8Array, maxWidth?: number): this {
    let b64: string;
    if (typeof data === 'string') {
      b64 = data;
    } else {
      b64 = bytesToBase64(data);
    }
    this.payload.commands?.push({ type: 'image', data: b64, max_width: maxWidth });
    return this;
  }

  // --- High-Level Structured Ticket Helpers (Backward Compatible) ---

  public title(title: string): this {
    this.payload.title = title;
    return this;
  }

  public subtitle(subtitle: string): this {
    this.payload.subtitle = subtitle;
    return this;
  }

  public address(address: string): this {
    this.payload.address = address;
    return this;
  }

  public meta(label: string, value: string): this {
    this.payload.metadata?.push({ label, value });
    return this;
  }

  public item(description: string, total: string, qty: string | number = 1, price?: string | number): this {
    this.payload.items?.push({
      description,
      total,
      qty: String(qty),
      price: price !== undefined ? String(price) : undefined,
    });
    return this;
  }

  public total(label: string, value: string): this {
    this.payload.totals?.push({ label, value });
    return this;
  }

  public footer(footer: string): this {
    this.payload.footer = footer;
    return this;
  }

  public toJSON(): TicketPayload {
    return this.payload;
  }

  public compile(dialect: Dialect = 'escpos'): Uint8Array {
    return compileTicket(this.payload, dialect);
  }

  public renderSvg(): string {
    return renderSvg(this.payload);
  }

  public renderHtml(): string {
    return renderHtml(this.payload);
  }
}

function bytesToBase64(bytes: Uint8Array): string {
  if (typeof btoa === 'function') {
    let binary = '';
    for (let i = 0; i < bytes.length; i++) {
      binary += String.fromCharCode(bytes[i]);
    }
    return btoa(binary);
  }
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
  let result = '';
  let i = 0;
  for (; i + 2 < bytes.length; i += 3) {
    const n = (bytes[i] << 16) | (bytes[i + 1] << 8) | bytes[i + 2];
    result += chars[(n >> 18) & 63] + chars[(n >> 12) & 63] + chars[(n >> 6) & 63] + chars[n & 63];
  }
  if (i < bytes.length) {
    const rem = bytes.length - i;
    const n = rem === 1 ? bytes[i] << 16 : (bytes[i] << 16) | (bytes[i + 1] << 8);
    result += chars[(n >> 18) & 63] + chars[(n >> 12) & 63];
    result += rem === 1 ? '==' : chars[(n >> 6) & 63] + '=';
  }
  return result;
}
