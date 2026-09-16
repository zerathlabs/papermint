import { useCallback, useEffect, useRef, useState } from 'react';
import type {
  Dialect,
  GatewayStatus,
  LabelDialect,
  LabelPayload,
  PrinterGatewayOptions,
  PrinterTarget,
  TicketPayload,
} from './ExpoPapermint.types';
import { compileTicket } from './index';
import { compileLabel } from './Label';

/**
 * Dispatches raw byte buffer to any supported printer target
 * (object with `.write(data)` or direct function callback).
 */
export async function sendToPrinter(printer: PrinterTarget, data: Uint8Array): Promise<void> {
  if (typeof printer === 'function') {
    await printer(data);
  } else if (printer && typeof printer.write === 'function') {
    await printer.write(data);
  } else {
    throw new Error('Invalid printer target: must provide a function or object with a .write() method');
  }
}

/**
 * Instantly formats and prints an order locally over Bluetooth or network printer (5ms, 100% offline).
 *
 * Designed for table-side or cashier ordering. Does not require internet connectivity.
 *
 * @param printer The target printer interface or write function.
 * @param receiptOrPayload A Receipt builder instance or a TicketPayload object.
 * @param dialect Thermal printer wire protocol ('escpos' or 'star'). Defaults to 'escpos'.
 */
export async function printLocalOrder(
  printer: PrinterTarget,
  receiptOrPayload: { compile?(dialect?: Dialect): Uint8Array } | TicketPayload,
  dialect: Dialect = 'escpos'
): Promise<void> {
  const bytes =
    typeof receiptOrPayload === 'object' && 'compile' in receiptOrPayload && typeof receiptOrPayload.compile === 'function'
      ? receiptOrPayload.compile(dialect)
      : compileTicket(receiptOrPayload as TicketPayload, dialect);

  await sendToPrinter(printer, bytes);
}

/**
 * Instantly formats and prints a 2D adhesive label locally over Bluetooth or USB.
 *
 * @param printer The target printer interface or write function.
 * @param labelOrPayload A Label builder instance or a LabelPayload object.
 * @param dialect Label dialect ('tspl' or 'zpl'). Defaults to 'tspl'.
 */
export async function printLocalLabel(
  printer: PrinterTarget,
  labelOrPayload: { compile?(dialect?: LabelDialect): Uint8Array } | LabelPayload,
  dialect: LabelDialect = 'tspl'
): Promise<void> {
  const bytes =
    typeof labelOrPayload === 'object' && 'compile' in labelOrPayload && typeof labelOrPayload.compile === 'function'
      ? labelOrPayload.compile(dialect)
      : compileLabel(labelOrPayload as LabelPayload, dialect);

  await sendToPrinter(printer, bytes);
}

/**
 * Standalone, headless Cloud Printing Gateway client with automatic reconnect.
 *
 * Can be used in React Native background tasks, Headless JS, or vanilla environments.
 */
export class PrinterGatewayClient {
  private ws: WebSocket | null = null;
  private reconnectTimer: any = null;
  private currentBackoff: number;
  private isDestroyed: boolean = false;
  private options: Required<Pick<PrinterGatewayOptions, 'autoReconnect' | 'reconnectInterval' | 'maxReconnectInterval'>> &
    PrinterGatewayOptions;
  private printer: PrinterTarget | null;
  private status: GatewayStatus;

  constructor(shopId: string, printer: PrinterTarget | null, options: PrinterGatewayOptions = {}) {
    this.printer = printer;
    this.options = {
      url: options.url || 'ws://127.0.0.1:8043/ws',
      shopId,
      token: options.token,
      autoReconnect: options.autoReconnect ?? true,
      reconnectInterval: options.reconnectInterval ?? 1000,
      maxReconnectInterval: options.maxReconnectInterval ?? 30000,
      onStatusChange: options.onStatusChange,
      onJobComplete: options.onJobComplete,
    };
    this.currentBackoff = this.options.reconnectInterval;
    this.status = {
      connected: false,
      connecting: false,
      shopId,
    };
  }

  public updatePrinter(printer: PrinterTarget | null): void {
    this.printer = printer;
  }

  public connect(): void {
    if (this.isDestroyed || !this.options.shopId || !this.options.url) return;

    this.cleanupSocket();
    this.updateStatus({ connecting: true, error: undefined });

    let targetUrl = this.options.url;
    if (this.options.shopId && !targetUrl.includes(this.options.shopId) && targetUrl.endsWith('/')) {
      targetUrl = `${targetUrl}${this.options.shopId}`;
    }

    try {
      const ws = new WebSocket(targetUrl);
      this.ws = ws;

      ws.onopen = () => {
        if (this.ws !== ws) return;
        this.currentBackoff = this.options.reconnectInterval;
        this.updateStatus({ connected: true, connecting: false, error: undefined });

        // Register with the cloud gateway
        const regMsg = {
          type: 'register',
          event: 'REGISTER',
          version: '0.2.3',
          shop_id: this.options.shopId,
          token: this.options.token,
          capabilities: ['escpos', 'star', 'tspl', 'zpl', 'raw'],
        };
        ws.send(JSON.stringify(regMsg));
      };

      ws.onmessage = async (event) => {
        if (this.ws !== ws) return;
        try {
          const data = JSON.parse(typeof event.data === 'string' ? event.data : '');
          await this.handleIncomingMessage(data);
        } catch (err: any) {
          console.warn('[expo-papermint] Failed to handle gateway message:', err);
        }
      };

      ws.onerror = (evt: any) => {
        if (this.ws !== ws) return;
        const errMsg = evt?.message || 'WebSocket error';
        this.updateStatus({ error: errMsg });
      };

      ws.onclose = () => {
        if (this.ws !== ws) return;
        this.updateStatus({ connected: false, connecting: false });
        this.scheduleReconnect();
      };
    } catch (err: any) {
      this.updateStatus({ connected: false, connecting: false, error: err?.message || 'Connection failed' });
      this.scheduleReconnect();
    }
  }

  private async handleIncomingMessage(data: any): Promise<void> {
    // 1. Keepalive Ping Handling
    if (data.type === 'ping' || data.event === 'PING') {
      const ts = data.timestamp || Date.now();
      this.sendSafe(
        JSON.stringify({
          type: 'pong',
          event: 'PONG',
          timestamp: ts,
        })
      );
      return;
    }

    // 2. Print Job Execution
    const isPrintJob = data.type === 'print_job' || data.event === 'PRINT_JOB' || data.orderId || data.job_id;
    if (isPrintJob) {
      const jobId = String(data.job_id || data.orderId || 'unknown');
      this.updateStatus({ lastJobId: jobId, lastJobTime: new Date() });

      if (!this.printer) {
        const err = 'No printer connected or configured on device';
        this.sendAck(jobId, false, 0, err);
        this.options.onJobComplete?.(jobId, false, err);
        return;
      }

      try {
        let wireBytes: Uint8Array;

        if (data.raw_bytes) {
          wireBytes = new Uint8Array(data.raw_bytes);
        } else if (data.wireBytes) {
          wireBytes = new Uint8Array(data.wireBytes);
        } else if (data.ticket) {
          const dialect: Dialect = data.dialect === 'star' ? 'star' : 'escpos';
          wireBytes = compileTicket(data.ticket, dialect);
        } else if (data.label) {
          const dialect: LabelDialect = data.dialect === 'zpl' ? 'zpl' : 'tspl';
          wireBytes = compileLabel(data.label, dialect);
        } else {
          throw new Error('Print job payload missing raw_bytes, wireBytes, ticket, or label specification');
        }

        await sendToPrinter(this.printer, wireBytes);

        this.sendAck(jobId, true, wireBytes.length);
        this.options.onJobComplete?.(jobId, true);
      } catch (err: any) {
        const errMsg = err?.message || String(err);
        this.sendAck(jobId, false, 0, errMsg);
        this.options.onJobComplete?.(jobId, false, errMsg);
      }
    }
  }

  private sendAck(jobId: string, success: boolean, bytesSent: number, error?: string): void {
    const ack = {
      type: 'ack',
      event: 'PRINT_ACK',
      job_id: jobId,
      orderId: jobId,
      success,
      status: success ? 'SUCCESS' : 'ERROR',
      bytes_sent: bytesSent,
      error: error || null,
    };
    this.sendSafe(JSON.stringify(ack));
  }

  private sendSafe(msg: string): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(msg);
    }
  }

  private scheduleReconnect(): void {
    if (this.isDestroyed || !this.options.autoReconnect) return;

    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
    }

    this.reconnectTimer = setTimeout(() => {
      this.connect();
    }, this.currentBackoff);

    this.currentBackoff = Math.min(this.currentBackoff * 2, this.options.maxReconnectInterval);
  }

  private updateStatus(partial: Partial<GatewayStatus>): void {
    this.status = { ...this.status, ...partial };
    this.options.onStatusChange?.(this.status);
  }

  private cleanupSocket(): void {
    if (this.ws) {
      try {
        this.ws.onopen = null;
        this.ws.onmessage = null;
        this.ws.onerror = null;
        this.ws.onclose = null;
        this.ws.close();
      } catch {
        // Ignored
      }
      this.ws = null;
    }
  }

  public disconnect(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    this.cleanupSocket();
    this.updateStatus({ connected: false, connecting: false });
  }

  public destroy(): void {
    this.isDestroyed = true;
    this.disconnect();
  }

  public getStatus(): GatewayStatus {
    return this.status;
  }
}

/**
 * React hook that connects an in-shop mobile device (cashier or counter phone)
 * to the Papermint Cloud WebSocket stream.
 *
 * Listens for remote print jobs dispatched from web dashboards or remote orders
 * and streams them directly to the in-shop Bluetooth or network thermal printer.
 *
 * @param shopId Unique store/shop identifier.
 * @param printer The target printer interface or write function (can be null if not yet connected).
 * @param options Additional gateway and reconnection parameters.
 * @returns Current gateway status and manual control handlers.
 */
export function usePrinterGateway(
  shopId: string,
  printer: PrinterTarget | null,
  options: PrinterGatewayOptions = {}
) {
  const [status, setStatus] = useState<GatewayStatus>({
    connected: false,
    connecting: false,
    shopId,
  });

  const clientRef = useRef<PrinterGatewayClient | null>(null);
  const optionsRef = useRef(options);
  optionsRef.current = options;

  useEffect(() => {
    if (!shopId) return;

    const client = new PrinterGatewayClient(shopId, printer, {
      ...optionsRef.current,
      onStatusChange: (newStatus) => {
        setStatus(newStatus);
        optionsRef.current.onStatusChange?.(newStatus);
      },
      onJobComplete: (jobId, success, err) => {
        optionsRef.current.onJobComplete?.(jobId, success, err);
      },
    });

    clientRef.current = client;
    client.connect();

    return () => {
      client.destroy();
      clientRef.current = null;
    };
  }, [shopId, options.url, options.token]);

  useEffect(() => {
    clientRef.current?.updatePrinter(printer);
  }, [printer]);

  const reconnect = useCallback(() => {
    clientRef.current?.connect();
  }, []);

  const disconnect = useCallback(() => {
    clientRef.current?.disconnect();
  }, []);

  return {
    status,
    isConnected: status.connected,
    isConnecting: status.connecting,
    lastJobId: status.lastJobId,
    lastJobTime: status.lastJobTime,
    error: status.error,
    reconnect,
    disconnect,
  };
}

