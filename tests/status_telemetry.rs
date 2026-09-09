use papermint::dialect::Dialect;
use papermint::{CoverStatus, DrawerStatus, EscPos, PaperStatus, Printer, Star};

#[test]
fn test_escpos_normal_status_ready() {
    let escpos = EscPos::new();
    // 4 bytes: [DLE EOT 1, DLE EOT 2, DLE EOT 3, DLE EOT 4]
    // All zero flags: online, drawer closed, cover closed, no errors, paper adequate
    let bytes = [0x00, 0x00, 0x00, 0x00];
    let status = escpos
        .parse_status_response(&bytes)
        .expect("parsing status failed");

    assert!(status.is_online);
    assert_eq!(status.cover, CoverStatus::Closed);
    assert_eq!(status.paper, PaperStatus::Adequate);
    assert_eq!(status.drawer, DrawerStatus::Closed);
    assert!(!status.cutter_error);
    assert!(!status.head_overheated);
    assert!(status.is_ready());
}

#[test]
fn test_escpos_drawer_open() {
    let escpos = EscPos::new();
    // Byte 0: Bit 2 (0x04) set -> Drawer Kick-out Pin 3 is open
    let bytes = [0x04, 0x00, 0x00, 0x00];
    let status = escpos.parse_status_response(&bytes).unwrap();

    assert_eq!(status.drawer, DrawerStatus::Open);
    assert!(status.is_online);
}

#[test]
fn test_escpos_cover_open() {
    let escpos = EscPos::new();
    // Byte 1: Bit 2 (0x04) set -> Cover is open
    let bytes = [0x00, 0x04, 0x00, 0x00];
    let status = escpos.parse_status_response(&bytes).unwrap();

    assert_eq!(status.cover, CoverStatus::Open);
    assert!(!status.is_ready());
}

#[test]
fn test_escpos_paper_near_end() {
    let escpos = EscPos::new();
    // Byte 3: Bits 2 and 3 (0x0C) set -> Paper near-end sensor triggered
    let bytes = [0x00, 0x00, 0x00, 0x0C];
    let status = escpos.parse_status_response(&bytes).unwrap();

    assert_eq!(status.paper, PaperStatus::NearEnd);
    assert!(
        status.is_ready(),
        "near-end paper warning should still be ready to print"
    );
}

#[test]
fn test_escpos_paper_empty() {
    let escpos = EscPos::new();
    // Byte 3: Bits 5 and 6 (0x60) set -> Paper roll sensor is empty
    let bytes = [0x00, 0x00, 0x00, 0x60];
    let status = escpos.parse_status_response(&bytes).unwrap();

    assert_eq!(status.paper, PaperStatus::Empty);
    assert!(!status.is_ready());
}

#[test]
fn test_escpos_cutter_error_and_head_overheat() {
    let escpos = EscPos::new();
    // Byte 2: Bit 3 (0x08) cutter error + Bit 6 (0x40) head overheat
    let bytes = [0x00, 0x00, 0x08 | 0x40, 0x00];
    let status = escpos.parse_status_response(&bytes).unwrap();

    assert!(status.cutter_error);
    assert!(status.head_overheated);
    assert!(!status.is_ready());
}

#[test]
fn test_star_enq_normal_status() {
    let star = Star::new();
    // Single byte ENQ response 0x00: online, paper adequate, cover closed, drawer closed
    let bytes = [0x00];
    let status = star.parse_status_response(&bytes).unwrap();

    assert!(status.is_online);
    assert_eq!(status.cover, CoverStatus::Closed);
    assert_eq!(status.paper, PaperStatus::Adequate);
    assert_eq!(status.drawer, DrawerStatus::Closed);
    assert!(status.is_ready());
}

#[test]
fn test_star_enq_paper_empty_and_cover_open() {
    let star = Star::new();
    // Bit 5 (0x20) cover open + Bit 6 (0x40) paper empty
    let bytes = [0x20 | 0x40];
    let status = star.parse_status_response(&bytes).unwrap();

    assert_eq!(status.cover, CoverStatus::Open);
    assert_eq!(status.paper, PaperStatus::Empty);
    assert!(!status.is_ready());
}

#[tokio::test]
async fn test_printer_query_status_roundtrip_escpos() {
    let mut printer = Printer::escpos_mock();

    // Inject mock DLE EOT response: drawer open (0x04) + near-end paper (0x0C)
    let mock_response = vec![0x04, 0x00, 0x00, 0x0C];
    printer.transport().set_read_response(mock_response);

    let status = printer
        .query_status()
        .await
        .expect("query_status should succeed");

    assert_eq!(status.drawer, DrawerStatus::Open);
    assert_eq!(status.paper, PaperStatus::NearEnd);
    assert_eq!(status.cover, CoverStatus::Closed);
    assert!(status.is_online);

    // Verify printer transmitted the DLE EOT 1..4 interrogation sequence:
    let transmitted = printer.transport().bytes();
    assert_eq!(
        transmitted,
        vec![
            0x10, 0x04, 0x01, 0x10, 0x04, 0x02, 0x10, 0x04, 0x03, 0x10, 0x04, 0x04
        ]
    );
}

#[tokio::test]
async fn test_printer_query_status_roundtrip_star() {
    let mut printer = Printer::star_mock();

    // Inject mock ENQ response: drawer open (0x04)
    printer.transport().set_read_response(vec![0x04]);

    let status = printer
        .query_status()
        .await
        .expect("query_status should succeed");

    assert_eq!(status.drawer, DrawerStatus::Open);
    assert_eq!(status.paper, PaperStatus::Adequate);

    // Verify Star query transmitted ENQ (0x05):
    let transmitted = printer.transport().bytes();
    assert_eq!(transmitted, vec![0x05]);
}

#[test]
fn test_escpos_real_hardware_fixed_bits_idle_ready() {
    let escpos = EscPos::new();
    // Genuine Epson TM-T88 hardware idle state with fixed bits 1 & 4 (0x12 = 00010010b) set on all 4 bytes:
    let bytes = [0x12, 0x12, 0x12, 0x12];
    let status = escpos
        .parse_status_response(&bytes)
        .expect("parsing real hardware frame failed");

    assert!(status.is_online);
    assert_eq!(status.cover, CoverStatus::Closed);
    assert_eq!(status.paper, PaperStatus::Adequate);
    assert_eq!(status.drawer, DrawerStatus::Closed);
    assert!(!status.cutter_error);
    assert!(!status.head_overheated);
    assert!(status.is_ready());
}

#[test]
fn test_escpos_real_hardware_fixed_bits_near_end_and_empty() {
    let escpos = EscPos::new();
    // Hardware near-end: Byte 4 has bits 1, 4 (0x12) + bits 2, 3 (0x0C) -> 0x1E
    let bytes_near = [0x12, 0x12, 0x12, 0x1E];
    let status_near = escpos.parse_status_response(&bytes_near).unwrap();
    assert_eq!(status_near.paper, PaperStatus::NearEnd);
    assert!(status_near.is_ready());

    // Hardware out of paper: Byte 4 has bits 1, 4 (0x12) + bits 5, 6 (0x60) -> 0x72
    let bytes_empty = [0x12, 0x12, 0x12, 0x72];
    let status_empty = escpos.parse_status_response(&bytes_empty).unwrap();
    assert_eq!(status_empty.paper, PaperStatus::Empty);
    assert!(!status_empty.is_ready());
}

#[tokio::test]
async fn test_printer_query_status_tcp_fragmentation_reassembly() {
    // Simulates TCP network delivery where 4-byte ESC/POS telemetry arrives in 2 separate network packets
    struct FragmentedMockTransport {
        chunks: Vec<Vec<u8>>,
        chunk_idx: usize,
    }

    #[async_trait::async_trait]
    impl papermint::transport::Transport for FragmentedMockTransport {
        async fn write_all(&mut self, _buf: &[u8]) -> papermint::error::Result<()> {
            Ok(())
        }
        async fn flush(&mut self) -> papermint::error::Result<()> {
            Ok(())
        }
        async fn read(&mut self, buf: &mut [u8]) -> papermint::error::Result<usize> {
            if self.chunk_idx < self.chunks.len() {
                let chunk = &self.chunks[self.chunk_idx];
                self.chunk_idx += 1;
                let to_copy = buf.len().min(chunk.len());
                buf[..to_copy].copy_from_slice(&chunk[..to_copy]);
                Ok(to_copy)
            } else {
                Ok(0)
            }
        }
    }

    let transport = FragmentedMockTransport {
        chunks: vec![
            vec![0x12],                    // Packet 1: First byte arrives alone
            vec![0x12, 0x12, 0x12 | 0x0C], // Packet 2: Remaining 3 bytes arrive (with paper near-end)
        ],
        chunk_idx: 0,
    };

    let mut printer = Printer::new(EscPos::new(), transport);
    let status = printer
        .query_status()
        .await
        .expect("query_status must reassemble fragmented packets");

    assert!(status.is_online);
    assert_eq!(status.paper, PaperStatus::NearEnd);
    assert_eq!(status.cover, CoverStatus::Closed);
    assert!(status.is_ready());
}
