import ExpoPapermintModule from './ExpoPapermintModule';
import type {
  LabelBarcodeType,
  LabelDialect,
  LabelPayload,
  LabelQrEcc,
} from './ExpoPapermint.types';

/**
 * Compiles a 2D label specification into TSPL-II or ZPL II binary wire bytes.
 *
 * @param label The label payload definition.
 * @param dialect Label printer dialect ('tspl' or 'zpl'). Defaults to 'tspl'.
 * @returns Uint8Array of binary wire bytes ready to transmit to the label printer.
 */
export function compileLabel(label: LabelPayload, dialect: LabelDialect = 'tspl'): Uint8Array {
  const jsonStr = JSON.stringify(label);
  return ExpoPapermintModule.compileLabel(jsonStr, dialect);
}

/**
 * Renders a 2D label specification into a virtual SVG vector graphic string.
 *
 * Features authentic die-cut rounded sticker canvas simulation, crisp vector barcodes,
 * and high-resolution typography.
 *
 * @param label The label payload definition.
 * @returns SVG string ready for rendering in React Native SVG or WebViews.
 */
export function renderLabelSvg(label: LabelPayload): string {
  const jsonStr = JSON.stringify(label);
  return ExpoPapermintModule.renderLabelSvg(jsonStr);
}

/**
 * Fluent 2D Label & Sticker Canvas Builder for mobile POS apps.
 *
 * Designed for barcode stickers, price tags, shelf labels, and shipping tags.
 * Supports dual-dialect compilation into TSPL-II (TSC, Xprinter, Munbyn) and
 * ZPL II (Zebra, Citizen, Godex), with instant virtual SVG previewing.
 */
export class Label {
  private payload: LabelPayload;

  constructor(widthMm: number = 50, heightMm: number = 30, dpi: number = 203) {
    this.payload = {
      width_mm: widthMm,
      height_mm: heightMm,
      dpi,
      elements: [],
      print_copies: 1,
    };
  }

  /**
   * Creates a new Label canvas instance.
   *
   * @param widthMm Physical label width in millimeters (e.g. 50, 76, 100).
   * @param heightMm Physical label height in millimeters (e.g. 30, 50, 150).
   * @param dpi Printhead resolution in DPI (default: 203).
   */
  public static create(widthMm: number = 50, heightMm: number = 30, dpi: number = 203): Label {
    return new Label(widthMm, heightMm, dpi);
  }

  /** Sets printhead resolution in dots per inch (default: 203). */
  public dpi(dpi: number): this {
    this.payload.dpi = dpi;
    return this;
  }

  /** Sets gap sensor distance and optional offset between labels in millimeters. */
  public gap(gapMm: number, _offsetMm: number = 0): this {
    this.payload.gap_mm = gapMm;
    return this;
  }

  /** Sets print speed in inches per second (typically 2 to 6). */
  public speed(speed: number): this {
    this.payload.speed = speed;
    return this;
  }

  /** Sets print darkness / density (0 to 15). */
  public density(density: number): this {
    this.payload.density = density;
    return this;
  }

  /** Sets print direction: false = Normal/Forward, true = Inverted. */
  public direction(_inverted: boolean = false): this {
    return this;
  }

  /** Appends text at absolute (x, y) dot coordinates with horizontal and vertical multipliers. */
  public text(
    x: number,
    y: number,
    content: string,
    xMult: number = 1,
    yMult: number = 1
  ): this {
    this.payload.elements?.push({
      type: 'text',
      x,
      y,
      content,
      x_mult: xMult,
      y_mult: yMult,
    });
    return this;
  }

  /** Appends a 1D barcode at (x, y). */
  public barcode(
    x: number,
    y: number,
    content: string,
    format: LabelBarcodeType = 'code128',
    height: number = 48,
    readable: boolean = true
  ): this {
    this.payload.elements?.push({
      type: 'barcode',
      x,
      y,
      content,
      format,
      height,
      readable,
    });
    return this;
  }

  /** Appends a 2D QR Code at (x, y). */
  public qr(
    x: number,
    y: number,
    content: string,
    cellWidth: number = 4,
    ecc: LabelQrEcc = 'M'
  ): this {
    this.payload.elements?.push({
      type: 'qr',
      x,
      y,
      content,
      cell_width: cellWidth,
      ecc,
    });
    return this;
  }

  /** Appends a rectangular bounding box at (x, y). */
  public box(x: number, y: number, width: number, height: number, thickness: number = 2): this {
    this.payload.elements?.push({
      type: 'box',
      x,
      y,
      width,
      height,
      thickness,
    });
    return this;
  }

  /** Appends a rectangular bounding box at (x, y) (alias for box). */
  public boxRect(x: number, y: number, width: number, height: number, thickness: number = 2): this {
    return this.box(x, y, width, height, thickness);
  }

  /** Appends a solid separator bar/line at (x, y). */
  public line(x: number, y: number, width: number, height: number): this {
    this.payload.elements?.push({
      type: 'line',
      x,
      y,
      width,
      height,
    });
    return this;
  }

  /** Inverts black/white pixels in a rectangular area (highlight badge). */
  public reverse(x: number, y: number, width: number, height: number): this {
    this.payload.elements?.push({
      type: 'reverse',
      x,
      y,
      width,
      height,
    });
    return this;
  }

  /** Sets number of copies to print. */
  public copies(count: number): this {
    this.payload.print_copies = count;
    return this;
  }

  /** Compiles the canvas into TSPL-II or ZPL II wire bytes. */
  public compile(dialect: LabelDialect = 'tspl'): Uint8Array {
    return compileLabel(this.payload, dialect);
  }

  /** Encodes the canvas into TSPL-II wire bytes (TSC, Xprinter, Rongta, Munbyn). */
  public encodeTspl(): Uint8Array {
    return this.compile('tspl');
  }

  /** Encodes the canvas into ZPL II wire bytes (Zebra, Citizen, Godex). */
  public encodeZpl(): Uint8Array {
    return this.compile('zpl');
  }

  /** Renders a virtual SVG sticker preview with rounded corners and authentic canvas dimensions. */
  public renderSvg(): string {
    return renderLabelSvg(this.payload);
  }

  /** Serializes the label canvas to a JSON-compatible payload object. */
  public toJSON(): LabelPayload {
    return this.payload;
  }
}

