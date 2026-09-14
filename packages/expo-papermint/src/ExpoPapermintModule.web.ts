import { registerWebModule, NativeModule } from 'expo';

class ExpoPapermintModule extends NativeModule<{}> {
  compileTicket(json: string, dialect: string): Uint8Array {
    console.warn(
      '[expo-papermint] compileTicket was called in a Web browser environment. ' +
      'Direct binary thermal printing is optimized for iOS and Android native runtimes.'
    );
    return new Uint8Array();
  }
}

export default registerWebModule(ExpoPapermintModule, 'ExpoPapermintModule');


