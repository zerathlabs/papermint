import { NativeModule, requireNativeModule } from 'expo';

declare class ExpoPapermintModule extends NativeModule<{}> {
  compileTicket(json: string, dialect: string): Uint8Array;
  renderSvg(json: string): string;
  renderHtml(json: string): string;
  compileLabel(json: string, dialect: string): Uint8Array;
  renderLabelSvg(json: string): string;
}

export default requireNativeModule<ExpoPapermintModule>('ExpoPapermint');


