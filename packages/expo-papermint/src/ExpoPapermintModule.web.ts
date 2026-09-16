import { registerWebModule, NativeModule } from 'expo';

class ExpoPapermintModule extends NativeModule<{}> {
  compileTicket(json: string, dialect: string): Uint8Array {
    console.warn(
      '[expo-papermint] compileTicket was called in a Web browser environment. ' +
      'Direct binary thermal printing is optimized for iOS and Android native runtimes.'
    );
    return new Uint8Array();
  }

  renderSvg(json: string): string {
    return '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 384 100"><text x="10" y="50" font-family="monospace">Receipt Preview</text></svg>';
  }

  renderHtml(json: string): string {
    return '<div style="font-family: monospace; padding: 12px; background: white; border: 1px solid #ddd;">Receipt Preview</div>';
  }

  compileLabel(json: string, dialect: string): Uint8Array {
    console.warn(
      '[expo-papermint] compileLabel was called in a Web browser environment. ' +
      'Direct binary thermal label printing is optimized for iOS and Android native runtimes.'
    );
    return new Uint8Array();
  }

  renderLabelSvg(json: string): string {
    return '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 384 240"><rect width="384" height="240" fill="white" rx="10" stroke="#ccc"/><text x="20" y="50" font-family="monospace">Label Preview</text></svg>';
  }
}

export default registerWebModule(ExpoPapermintModule, 'ExpoPapermintModule');


