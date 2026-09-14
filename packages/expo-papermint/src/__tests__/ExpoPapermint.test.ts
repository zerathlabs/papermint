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
});

