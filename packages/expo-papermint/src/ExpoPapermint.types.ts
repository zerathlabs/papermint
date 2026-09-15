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
