export type Dialect = 'escpos' | 'star';
export type PaperWidth = '80mm' | '58mm';
export type CutMode = 'full' | 'partial' | 'none';

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
