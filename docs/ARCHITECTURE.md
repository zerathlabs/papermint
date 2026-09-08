# 🏛️ Architecture & Internal Design

`papermint` is designed to solve a fundamental problem in POS development: **vendor lock-in and ad-hoc byte concatenation**.

Traditional receipt printing libraries directly emit ESC/POS bytes while constructing the receipt layout. This causes three major architectural problems:
1. **Vendor Coupling**: Switching from an Epson printer to a Star Micronics printer requires rewriting the entire printing pipeline.
2. **Untestable Code**: You cannot unit-test receipt formatting without physical hardware attached.
3. **Encoding Fragility**: Multi-byte UTF-8 characters (accents, currency symbols like `€`, non-Latin text) break column alignment when calculated using byte lengths.

`papermint` solves this with a **three-layer decoupled architecture**.

---

## 1. The Three Layers

```text
┌─────────────────────────────────────────────────────────┐
│  Layer 3: Layout Engine (Receipt Builder)               │
│  - Fluent API, Unicode-aware columns, tables, styling   │
└──────────────────────────┬──────────────────────────────┘
                           │ produces Vec<Command> (Command IR)
┌──────────────────────────▼──────────────────────────────┐
│  Layer 2: Dialect Encoders                              │
│  - EscPos (Epson, Bixolon, Citizen, Xprinter)           │
│  - Star (Star Micronics TSP100, TSP650, mC-Print)       │
└──────────────────────────┬──────────────────────────────┘
                           │ emits wire bytes (Vec<u8>)
┌──────────────────────────▼──────────────────────────────┐
│  Layer 1: Transports                                    │
│  - Async TCP (Port 9100 RAW network printing)           │
│  - VecSink (In-memory testing and mock execution)       │
└─────────────────────────────────────────────────────────┘
```

### Layer 3: Layout Engine (`src/layout/`)
- **`Receipt`**: A fluent, chainable builder that produces an abstract stream of `Command` variants (`Vec<Command>`).
- **`column.rs`**: Implements visual column mathematics. It uses `str.chars().count()` instead of `str.len()` so that multi-byte Unicode characters are accurately aligned.

### Layer 2: Dialect Encoders (`src/dialect/`)
- **`Dialect` Trait**:
  ```rust
  pub trait Dialect: Send + Sync {
      fn name(&self) -> &'static str;
      fn encode(&self, command: &Command, buf: &mut Vec<u8>) -> Result<()>;
  }
  ```
- **`EscPos`**: Encodes standard Epson ESC/POS wire protocols.
- **`Star`**: Encodes Star Micronics Line Mode / StarPRNT wire protocols.
- **Atomic State Tracking**: State that alters downstream byte formatting (such as character width/height scaling) is tracked using `AtomicU8`, allowing the dialect to remain thread-safe (`&self`) without internal mutability locks.

### Layer 1: Transports (`src/transport/`)
- **`Transport` Trait**:
  ```rust
  #[async_trait]
  pub trait Transport: Send {
      async fn write(&mut self, data: &[u8]) -> Result<()>;
  }
  ```
- **`TcpTransport`**: Asynchronous TCP client configured with connect timeouts, write timeouts, and automatic reconnection.
- **`VecSink`**: In-memory test sink that captures raw bytes into an internal buffer.

---

## 2. The Command Intermediate Representation (IR)

The central abstraction in `papermint` is [`Command`](file:///src/command.rs):

```rust
pub enum Command {
    Init,
    Text(String),
    Feed(u8),
    Cut(CutMode),
    Align(Alignment),
    Bold(bool),
    Underline(UnderlineMode),
    Invert(bool),
    DoubleHeight(bool),
    DoubleWidth(bool),
    Font(FontFamily),
    Barcode(BarcodeData),
    QrCode(QrData),
    Image(ImageData),
    DrawerKick(DrawerPin),
    Beep { count: u8, duration: u8 },
    LineSpacing(Option<u8>),
    UpsideDown(bool),
    CodePage(u8),
    Raw(Vec<u8>),
}
```

### Why this design matters:
1. **Write Once, Print Anywhere**: Construct the receipt layout once. The caller chooses at runtime whether to send it to an `EscPos` printer, a `Star` printer, or an in-memory `VecSink`.
2. **Zero Overhead**: The encoder pre-allocates buffer capacity (`with_capacity(commands.len() * 16)`) and appends bytes directly, avoiding temporary string or buffer allocations.
3. **Extensibility**: Adding support for a new vendor dialect simply requires implementing the `Dialect` trait for that vendor's byte sequences without changing any receipt layout logic.
