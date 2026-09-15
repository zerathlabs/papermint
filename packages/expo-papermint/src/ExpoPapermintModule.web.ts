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
}

export default registerWebModule(ExpoPapermintModule, 'ExpoPapermintModule');


