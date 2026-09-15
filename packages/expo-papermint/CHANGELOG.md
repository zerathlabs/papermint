# Changelog

## [0.2.3] - 2026-09-15

### 🎉 New Features
- **100% Feature Parity with Rust Core & Node.js**:
  - **Virtual Previews (`renderSvg`, `renderHtml`)**: Preview receipts on-screen in SVG or HTML before printing.
  - **Granular Typography**: Added `.bold()`, `.underline()`, `.invert()`, `.doubleSize()`, `.doubleWidth()`, `.doubleHeight()`, `.left()`, `.center()`, `.right()`, `.align()`.
  - **Multi-Column Tables**: Added `.setColumns()`, `.row()`, `.tableHeader()`, and `.clearColumns()` supporting fixed-width and fractional column layouts.
  - **Rich Dividers**: Added `.dividerDouble()`, `.dividerDotted()`, `.dividerDashed()`, and `.dividerPattern()`.
  - **Hardware Controls**: Added `.openDrawer()`, `.beep()`, `.cut()`, `.feed()`.
  - **Image Printing**: Added `.image(base64)` supporting portable Base64-encoded PNG/JPEG images without native image library dependencies.
  - **Code Pages**: Added `.codePage()` supporting international character sets and Arabic Windows-1256.
  - **Raw Commands**: Added `.raw(bytes)` to inject custom hardware printer escape sequences.

### 🐛 Bug Fixes
- Fixed Arabic Windows-1256 (WPC1256) encoding to match official Unicode Consortium standards.
- Added pure JavaScript Base64 converter for React Native environments where Node's `Buffer` is not globally available.

## [0.2.2] - 2026-09-14
- Initial official release of `expo-papermint`.
- Zero-copy native marshaling for Android (JNI) and iOS (Swift).
- Fluent `Receipt` builder, `compileTicket()`, and `generateZatcaQr()` support.
