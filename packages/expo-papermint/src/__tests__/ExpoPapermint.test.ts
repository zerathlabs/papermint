import { Receipt, generateZatcaQr } from '../index';

describe('expo-papermint', () => {
  it('builds ticket JSON matching papermint schema', () => {
    const receipt = Receipt.create('80mm')
      .title('COFFEE SHOP')
      .subtitle('Downtown')
      .meta('Order #', '101')
      .item('Espresso', '3.50', 1)
      .item('Croissant', '7.00', 2, '3.50')
      .total('Grand Total', '10.50')
      .footer('Have a nice day')
      .openDrawer()
      .beep(1)
      .cut('full');

    const payload = receipt.toJSON();
    expect(payload.paper_width).toBe('80mm');
    expect(payload.title).toBe('COFFEE SHOP');
    expect(payload.items?.length).toBe(2);
    expect(payload.totals?.[0].value).toBe('10.50');
    expect(payload.open_drawer).toBe(true);
    expect(payload.beep).toBe(1);
    expect(payload.cut_mode).toBe('full');
  });

  it('generates valid ZATCA Phase 1 & 2 TLV QR base64 string', () => {
    const qr = generateZatcaQr({
      sellerName: 'Papermint Coffee',
      vatNumber: '300000000000003',
      timestamp: '2026-09-14T12:00:00Z',
      totalAmount: '10.50',
      taxAmount: '1.50',
    });

    expect(typeof qr).toBe('string');
    expect(qr.length).toBeGreaterThan(20);
  });

  describe('Label Builder', () => {
    it('builds 2D label JSON payload matching papermint-label schema', () => {
      const label = Label.create(50, 30, 203)
        .gap(3)
        .speed(4)
        .density(10)
        .text(10, 15, 'ORGANIC MATCHA', 2, 2)
        .barcode(10, 45, 'MATCHA-001', 'code128', 50, true)
        .qr(150, 45, 'https://example.com/batch/99', 4, 'M')
        .box(5, 5, 200, 100, 2)
        .line(5, 40, 200, 2)
        .reverse(5, 5, 200, 15)
        .copies(2);

      const payload = label.toJSON();
      expect(payload.width_mm).toBe(50);
      expect(payload.height_mm).toBe(30);
      expect(payload.dpi).toBe(203);
      expect(payload.gap_mm).toBe(3);
      expect(payload.speed).toBe(4);
      expect(payload.density).toBe(10);
      expect(payload.print_copies).toBe(2);
      expect(payload.elements?.length).toBe(6);

      const textEl = payload.elements?.[0];
      expect(textEl?.type).toBe('text');
      if (textEl?.type === 'text') {
        expect(textEl.content).toBe('ORGANIC MATCHA');
        expect(textEl.x_mult).toBe(2);
      }

      const barEl = payload.elements?.[1];
      expect(barEl?.type).toBe('barcode');
      if (barEl?.type === 'barcode') {
        expect(barEl.format).toBe('code128');
        expect(barEl.readable).toBe(true);
      }
    });

    it('supports alias boxRect and compiles without error', () => {
      const label = Label.create(40, 25).boxRect(2, 2, 80, 40, 1);
      const payload = label.toJSON();
      expect(payload.elements?.[0].type).toBe('box');
    });
  });

  describe('Local Offline Printing Helpers', () => {
    it('dispatches compiled receipt bytes directly to printer target', async () => {
      const mockWrite = jest.fn().mockResolvedValue(undefined);
      const printer = { write: mockWrite };

      const receipt = Receipt.create('58mm').textLn('Table 4 Espresso');
      await printLocalOrder(printer, receipt);

      expect(mockWrite).toHaveBeenCalledTimes(1);
      expect(mockWrite.mock.calls[0][0]).toBeInstanceOf(Uint8Array);
    });

    it('dispatches compiled label bytes directly to printer function target', async () => {
      const mockPrinterFn = jest.fn().mockResolvedValue(undefined);

      const label = Label.create(50, 30).text(10, 10, 'Test Tag');
      await printLocalLabel(mockPrinterFn, label, 'tspl');

      expect(mockPrinterFn).toHaveBeenCalledTimes(1);
      expect(mockPrinterFn.mock.calls[0][0]).toBeInstanceOf(Uint8Array);
    });
  });

  describe('PrinterGatewayClient', () => {
    it('initializes status correctly with shopId', () => {
      const client = new PrinterGatewayClient('shop_downtown_01', null, {
        url: 'ws://localhost:8043/ws',
        autoReconnect: false,
      });

      const status = client.getStatus();
      expect(status.shopId).toBe('shop_downtown_01');
      expect(status.connected).toBe(false);
      expect(status.connecting).toBe(false);
      client.destroy();
    });
  });
});

