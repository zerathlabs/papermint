export type Dialect = 'escpos' | 'star';
export type PaperWidth = '80mm' | '58mm';
export type CutMode = 'full' | 'partial' | 'none';
export type TextAlignment = 'left' | 'center' | 'right';

/** Configuration options for table columns. */
export interface TableColumnOptions {
  /** Fixed character width. */
  widthFixed?: number;
  /** Simple fixed character width shorthand (alias for widthFixed). */
  width?: number;
  /** Proportional fraction of total columns (0.0 .. 1.0). */
  widthFraction?: number;
  /** Text alignment: "left", "center", or "right". */
  align?: TextAlignment;
  /** Text alignment alias: "left", "center", or "right" (alias for align). */
  alignment?: TextAlignment;
}

export type ReceiptCommand =
  | {
      type: 'text';
      text: string;
      align?: TextAlignment;
      bold?: boolean;
      underline?: boolean;
      invert?: boolean;
      double_width?: boolean;
      double_height?: boolean;
      double_size?: boolean;
    }
  | {
      type: 'text_ln';
      text: string;
      align?: TextAlignment;
      bold?: boolean;
      underline?: boolean;
      invert?: boolean;
      double_width?: boolean;
      double_height?: boolean;
      double_size?: boolean;
    }
  | { type: 'feed'; lines?: number }
  | { type: 'align'; align: TextAlignment }
  | { type: 'bold'; enable: boolean }
  | { type: 'underline'; enable: boolean; mode?: 'single' | 'double' }
  | { type: 'invert'; enable: boolean }
  | { type: 'double_size'; enable: boolean }
  | { type: 'double_width'; enable: boolean }
  | { type: 'double_height'; enable: boolean }
  | { type: 'divider'; style?: string; variant?: 'single' | 'double' | 'dotted' | 'dashed'; pattern?: string }
  | { type: 'two_column'; left: string; right: string }
  | { type: 'three_column'; left: string; center: string; right: string }
  | { type: 'table_header'; headers: string[]; columns: TableColumnOptions[] }
  | { type: 'set_columns'; columns: TableColumnOptions[] }
  | { type: 'row'; cells: string[] }
  | { type: 'clear_columns' }
  | { type: 'qr'; content: string }
  | { type: 'barcode'; content: string }
  | { type: 'code_page'; page: string | number }
  | { type: 'image'; data: string; max_width?: number }
  | { type: 'raw'; bytes?: number[]; base64?: string }
  | { type: 'open_drawer' }
  | { type: 'beep'; count?: number; duration?: number }
  | { type: 'cut'; mode?: CutMode };

export interface TicketItem {
  description: string;
  total: string;
  qty?: string | number;
  price?: string | number;
}

export interface TicketMetadata {
  label: string;
  value: string;
}

export interface TicketPayload {
  paper_width?: PaperWidth;
  code_page?: string | number;
  commands?: ReceiptCommand[];
  title?: string;
  subtitle?: string;
  address?: string;
  metadata?: TicketMetadata[];
  items?: TicketItem[];
  totals?: TicketMetadata[];
  qr?: string;
  barcode?: string;
  footer?: string;
  cut_mode?: CutMode;
  open_drawer?: boolean;
  beep?: number;
  divider_style?: string;
}

export interface ZatcaQrParams {
  sellerName: string;
  vatNumber: string;
  timestamp: string;
  totalAmount: string;
  taxAmount: string;
}

/* 2D Adhesive Label Types (TSPL & ZPL II) */

export type LabelDialect = 'tspl' | 'zpl';

export type LabelBarcodeType = 'code128' | 'code39' | 'ean13' | 'ean8' | 'upca' | 'itf';

export type LabelQrEcc = 'L' | 'M' | 'Q' | 'H';

export type LabelElement =
  | {
      type: 'text';
      x: number;
      y: number;
      content: string;
      x_mult?: number;
      y_mult?: number;
    }
  | {
      type: 'barcode';
      x: number;
      y: number;
      content: string;
      format?: LabelBarcodeType;
      height?: number;
      readable?: boolean;
    }
  | {
      type: 'qr';
      x: number;
      y: number;
      content: string;
      cell_width?: number;
      ecc?: LabelQrEcc;
    }
  | {
      type: 'box';
      x: number;
      y: number;
      width: number;
      height: number;
      thickness?: number;
    }
  | {
      type: 'line';
      x: number;
      y: number;
      width: number;
      height: number;
    }
  | {
      type: 'reverse';
      x: number;
      y: number;
      width: number;
      height: number;
    };

export interface LabelPayload {
  width_mm?: number;
  height_mm?: number;
  dpi?: number;
  gap_mm?: number;
  speed?: number;
  density?: number;
  elements?: LabelElement[];
  print_copies?: number;
}

/* Cloud Gateway & Hardware Printer Types */

/** Interface for any connected Bluetooth or network printer writer. */
export interface PrinterWritable {
  write(data: Uint8Array): Promise<unknown> | unknown;
}

/** Supported printer targets: an object with .write() or a direct function callback. */
export type PrinterTarget = PrinterWritable | ((data: Uint8Array) => Promise<unknown> | unknown);

/** Connection and telemetry status of the In-Shop Cloud Gateway. */
export interface GatewayStatus {
  connected: boolean;
  connecting: boolean;
  shopId?: string;
  lastJobId?: string;
  lastJobTime?: Date;
  error?: string;
}

/** Options for configuring the In-Shop Cloud Gateway hook. */
export interface PrinterGatewayOptions {
  /** Gateway WebSocket URL (e.g. wss://api.yourdomain.com/ws/printer or ws://localhost:8043/ws). */
  url?: string;
  /** Unique merchant/store identifier. */
  shopId?: string;
  /** Optional JWT or API bearer token for authentication. */
  token?: string;
  /** Automatically reconnect if WebSocket drops. Defaults to true. */
  autoReconnect?: boolean;
  /** Initial reconnection delay in milliseconds. Defaults to 1000. */
  reconnectInterval?: number;
  /** Maximum reconnection backoff delay in milliseconds. Defaults to 30000. */
  maxReconnectInterval?: number;
  /** Callback triggered when connection state updates. */
  onStatusChange?: (status: GatewayStatus) => void;
  /** Callback triggered when an incoming print job finishes executing. */
  onJobComplete?: (jobId: string, success: boolean, error?: string) => void;
}

