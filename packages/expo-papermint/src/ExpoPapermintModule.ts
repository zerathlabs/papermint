import { NativeModule, requireNativeModule } from 'expo';

declare class ExpoPapermintModule extends NativeModule<{}> {
  compileTicket(json: string, dialect: string): Uint8Array;
}

export default requireNativeModule<ExpoPapermintModule>('ExpoPapermint');


