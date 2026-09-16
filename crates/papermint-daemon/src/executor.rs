//! Print job execution and printer hardware status query engine.

use std::net::SocketAddr;
use std::time::Duration;

use papermint::{
    Alignment, CoverStatus, DrawerStatus, EscPos, PaperStatus, PaperWidth, Printer, Receipt, Star,
    TableColumn,
};

#[cfg(feature = "serial")]
use papermint::SerialTransport;
#[cfg(feature = "usb")]
use papermint::UsbTransport;

use crate::models::{
    DialectChoice, PrintRequest, PrintResponse, StatusRequest, StatusResponse, TargetConfig,
    TicketPayload,
};

/// Executes a print request against the specified target (mock, TCP, USB, or serial).
pub async fn execute_print(req: PrintRequest) -> Result<PrintResponse, String> {
    let paper_width = match req.paper_width.as_str() {
        "58mm" | "58" | "Mm58" => PaperWidth::Mm58,
        _ => PaperWidth::Mm80,
    };

    let receipt = if let Some(ticket) = req.ticket {
        build_receipt_from_ticket(paper_width, ticket)
    } else {
        Receipt::new(paper_width).init()
    };

    match req.target {
        TargetConfig::Mock => {
            let bytes = match req.dialect {
                DialectChoice::Star => {
                    let mut printer = Printer::star_mock();
                    if let Some(raw) = req.raw_bytes {
                        use papermint::transport::Transport;
                        printer
                            .transport_mut()
                            .write_all(&raw)
                            .await
                            .map_err(|e| e.to_string())?;
                    } else {
                        printer.print(&receipt).await.map_err(|e| e.to_string())?;
                    }
                    printer.transport().bytes()
                }
                DialectChoice::EscPos => {
                    let mut printer = Printer::escpos_mock();
                    if let Some(raw) = req.raw_bytes {
                        use papermint::transport::Transport;
                        printer
                            .transport_mut()
                            .write_all(&raw)
                            .await
                            .map_err(|e| e.to_string())?;
                    } else {
                        printer.print(&receipt).await.map_err(|e| e.to_string())?;
                    }
                    printer.transport().bytes()
                }
            };

            let preview = String::from_utf8_lossy(&bytes)
                .chars()
                .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
                .collect();

            Ok(PrintResponse {
                success: true,
                bytes_sent: bytes.len(),
                mock_preview: Some(preview),
            })
        }
        TargetConfig::Tcp { address } => {
            use papermint::transport::Transport;
            let sock: SocketAddr = address
                .parse()
                .map_err(|e| format!("invalid TCP socket address: {e}"))?;
            let mut transport = papermint::TcpTransport::new(sock)
                .with_connect_timeout(Duration::from_secs(3))
                .with_write_timeout(Duration::from_secs(5));

            let bytes_sent = if let Some(raw) = req.raw_bytes {
                let len = raw.len();
                transport.write_all(&raw).await.map_err(|e| e.to_string())?;
                len
            } else {
                match req.dialect {
                    DialectChoice::Star => {
                        let mut printer = Printer::new(Star::new(), transport);
                        printer.print(&receipt).await.map_err(|e| e.to_string())?;
                    }
                    DialectChoice::EscPos => {
                        let mut printer = Printer::new(EscPos::new(), transport);
                        printer.print(&receipt).await.map_err(|e| e.to_string())?;
                    }
                }
                0
            };

            Ok(PrintResponse {
                success: true,
                bytes_sent,
                mock_preview: None,
            })
        }
        TargetConfig::Serial { port, baud } => {
            #[cfg(feature = "serial")]
            {
                use papermint::transport::Transport;
                let b = baud.unwrap_or(19200);
                let mut transport = SerialTransport::new(&port, b);
                let bytes_sent = if let Some(raw) = req.raw_bytes {
                    let len = raw.len();
                    transport.write_all(&raw).await.map_err(|e| e.to_string())?;
                    len
                } else {
                    match req.dialect {
                        DialectChoice::Star => {
                            let mut printer = Printer::new(Star::new(), transport);
                            printer.print(&receipt).await.map_err(|e| e.to_string())?;
                        }
                        DialectChoice::EscPos => {
                            let mut printer = Printer::new(EscPos::new(), transport);
                            printer.print(&receipt).await.map_err(|e| e.to_string())?;
                        }
                    }
                    0
                };
                Ok(PrintResponse {
                    success: true,
                    bytes_sent,
                    mock_preview: None,
                })
            }
            #[cfg(not(feature = "serial"))]
            {
                let _ = (port, baud);
                Err("Serial feature disabled".into())
            }
        }
        TargetConfig::Usb {
            vendor_id,
            product_id,
        } => {
            #[cfg(feature = "usb")]
            {
                use papermint::transport::Transport;
                let mut transport = UsbTransport::from_vid_pid(vendor_id, product_id);
                let bytes_sent = if let Some(raw) = req.raw_bytes {
                    let len = raw.len();
                    transport.write_all(&raw).await.map_err(|e| e.to_string())?;
                    len
                } else {
                    match req.dialect {
                        DialectChoice::Star => {
                            let mut printer = Printer::new(Star::new(), transport);
                            printer.print(&receipt).await.map_err(|e| e.to_string())?;
                        }
                        DialectChoice::EscPos => {
                            let mut printer = Printer::new(EscPos::new(), transport);
                            printer.print(&receipt).await.map_err(|e| e.to_string())?;
                        }
                    }
                    0
                };
                Ok(PrintResponse {
                    success: true,
                    bytes_sent,
                    mock_preview: None,
                })
            }
            #[cfg(not(feature = "usb"))]
            {
                let _ = (vendor_id, product_id);
                Err("USB feature disabled".into())
            }
        }
    }
}

/// Queries printer status from the specified target (Mock or TCP).
pub async fn execute_status(req: StatusRequest) -> Result<StatusResponse, String> {
    match req.target {
        TargetConfig::Mock => {
            let status = match req.dialect {
                DialectChoice::Star => {
                    let mut printer = Printer::star_mock();
                    printer.transport().set_read_response(vec![0x12]);
                    printer.query_status().await.map_err(|e| e.to_string())?
                }
                DialectChoice::EscPos => {
                    let mut printer = Printer::escpos_mock();
                    printer.transport().set_read_response(vec![0x12]);
                    printer.query_status().await.map_err(|e| e.to_string())?
                }
            };

            Ok(map_status(status))
        }
        TargetConfig::Tcp { address } => {
            let sock: SocketAddr = address
                .parse()
                .map_err(|e| format!("invalid TCP socket address: {e}"))?;
            let transport = papermint::TcpTransport::new(sock)
                .with_connect_timeout(Duration::from_secs(3))
                .with_read_timeout(Duration::from_secs(3));

            let status = match req.dialect {
                DialectChoice::Star => {
                    let mut printer = Printer::new(Star::new(), transport);
                    printer.query_status().await.map_err(|e| e.to_string())?
                }
                DialectChoice::EscPos => {
                    let mut printer = Printer::new(EscPos::new(), transport);
                    printer.query_status().await.map_err(|e| e.to_string())?
                }
            };
            Ok(map_status(status))
        }
        _ => Err("Real-time status query currently only supported over TCP and Mock".into()),
    }
}

pub fn build_receipt_from_ticket(paper_width: PaperWidth, ticket: TicketPayload) -> Receipt {
    let mut receipt = Receipt::new(paper_width).init();

    if let Some(title) = ticket.title {
        receipt = receipt
            .center()
            .bold(true)
            .double_size(true)
            .text_ln(title)
            .double_size(false)
            .bold(false);
    }
    if let Some(sub) = ticket.subtitle {
        receipt = receipt.center().text_ln(sub);
    }
    if let Some(addr) = ticket.address {
        receipt = receipt.center().text_ln(addr);
    }

    let divider_style = ticket.divider_style.as_deref();
    let apply_divider = |r: Receipt, style: Option<&str>| match style {
        Some("double") => r.divider_double(),
        Some("dotted") => r.divider_dotted(),
        Some("dashed") => r.divider_dashed(),
        Some(pat) if pat.len() > 1 => r.divider_pattern(pat),
        Some(single) if single.len() == 1 => r.divider(single.chars().next().unwrap_or('-')),
        _ => r.divider('-'),
    };

    if let Some(meta) = ticket.metadata {
        receipt = apply_divider(receipt, divider_style);
        for m in meta {
            receipt = receipt.two_column(m.label, m.value);
        }
    }

    if let Some(items) = ticket.items {
        receipt = apply_divider(receipt, divider_style.or(Some("double")));
        let columns = [
            TableColumn::fixed(4, Alignment::Left),
            TableColumn::fraction(0.52, Alignment::Left),
            TableColumn::fraction(0.20, Alignment::Right),
            TableColumn::fraction(0.24, Alignment::Right),
        ];
        receipt = receipt.table_header(&["QTY", "ITEM", "PRICE", "TOTAL"], &columns);
        receipt = apply_divider(receipt, divider_style);
        for item in items {
            let unit = item.price.unwrap_or_default();
            receipt = receipt.row(&[&item.qty, &item.description, &unit, &item.total]);
        }
    }

    if let Some(totals) = ticket.totals {
        receipt = apply_divider(receipt, divider_style.or(Some("double")));
        for t in totals {
            receipt = receipt.two_column(t.label, t.value);
        }
    }

    if let Some(qr) = ticket.qr {
        receipt = receipt.feed(1).center().qr(qr);
    }

    if let Some(barcode) = ticket.barcode {
        receipt = receipt.feed(1).center().barcode_128(barcode);
    }

    if let Some(footer) = ticket.footer {
        receipt = receipt.feed(1).center().text_ln(footer);
    }

    if let Some(b) = ticket.beep {
        receipt = receipt.beep(b, 2);
    }

    if ticket.open_drawer.unwrap_or(false) {
        receipt = receipt.open_drawer();
    }

    match ticket.cut_mode.as_deref() {
        Some("partial") => receipt.feed(2).cut_partial(),
        Some("none") => receipt.feed(2),
        _ => receipt.feed(3).cut_full(),
    }
}

pub fn map_status(s: papermint::PrinterStatus) -> StatusResponse {
    StatusResponse {
        is_online: s.is_online,
        is_ready: s.is_ready(),
        paper: match s.paper {
            PaperStatus::Adequate => "Adequate".into(),
            PaperStatus::NearEnd => "NearEnd".into(),
            PaperStatus::Empty => "Empty".into(),
        },
        cover: match s.cover {
            CoverStatus::Closed => "Closed".into(),
            CoverStatus::Open => "Open".into(),
        },
        drawer: match s.drawer {
            DrawerStatus::Closed => "Closed".into(),
            DrawerStatus::Open => "Open".into(),
        },
        cutter_error: s.cutter_error,
        head_overheated: s.head_overheated,
    }
}
