// Reexport the native module. On web, it will be resolved to ExpoPapermintModule.web.ts
// and on native platforms to ExpoPapermintModule.ts
export { default as ExpoPapermintModule } from './ExpoPapermintModule';
export { default } from './ExpoPapermintModule';
import ExpoPapermintModule from './ExpoPapermintModule';
import {
  Dialect,
  PaperWidth,
  TicketPayload,
  ZatcaQrParams,
} from './ExpoPapermint.types';

export * from './ExpoPapermint.types';


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
 * Example:
 * ```ts
 * const bytes = Receipt.create('80mm')
 *   .title('MY COFFEE SHOP')
 *   .subtitle('Branch #101')
 *   .meta('Order #', '9842')
 *   .item('Espresso Single', '$3.50', 1)
 *   .item('Croissant Butter', '$8.00', 2, '$4.00')
 *   .total('Grand Total', '$11.50')
 *   .qr('https://coffeeshop.com/order/9842')
 *   .cut()
 *   .compile('escpos');
 * ```
 */
export class Receipt {
  private payload: TicketPayload;

  constructor(paperWidth: PaperWidth = '80mm') {
    this.payload = {
      paper_width: paperWidth,
      metadata: [],
      items: [],
      totals: [],
    };
  }

  public static create(paperWidth: PaperWidth = '80mm'): Receipt {
    return new Receipt(paperWidth);
  }

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

  public qr(content: string): this {
    this.payload.qr = content;
    return this;
  }

  public barcode(code: string): this {
    this.payload.barcode = code;
    return this;
  }

  public footer(footer: string): this {
    this.payload.footer = footer;
    return this;
  }

  public divider(style: string = '-'): this {
    this.payload.divider_style = style;
    return this;
  }

  public openDrawer(): this {
    this.payload.open_drawer = true;
    return this;
  }

  public beep(count: number = 1): this {
    this.payload.beep = count;
    return this;
  }

  public cut(mode: 'full' | 'partial' | 'none' = 'full'): this {
    this.payload.cut_mode = mode;
    return this;
  }

  public toJSON(): TicketPayload {
    return this.payload;
  }

  public compile(dialect: Dialect = 'escpos'): Uint8Array {
    return compileTicket(this.payload, dialect);
  }
}

