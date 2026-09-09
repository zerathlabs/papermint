//! papermintd — Lightweight native HTTP print sidecar daemon.
//!
//! Provides a high-performance local microservice for streaming POS print jobs,
//! checking real-time sensor telemetry, and enumerating hardware devices over HTTP/REST.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

use papermint::{
    Alignment, CoverStatus, DrawerStatus, EscPos, PaperStatus, PaperWidth, Printer, Receipt, Star,
    TableColumn,
};

#[cfg(feature = "serial")]
use papermint::SerialTransport;
#[cfg(feature = "usb")]
use papermint::UsbTransport;

/// Daemon application state.
pub struct AppState {
    pub start_time: Instant,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "papermintd=info,tower_http=info".into()),
        )
        .init();

    let port: u16 = std::env::var("PORT")
        .or_else(|_| std::env::var("PAPERMINTD_PORT"))
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let host = std::env::var("HOST")
        .or_else(|_| std::env::var("PAPERMINTD_HOST"))
        .unwrap_or_else(|_| "127.0.0.1".to_string());

    let state = Arc::new(AppState {
        start_time: Instant::now(),
    });

    let app = create_router(state);

    let bind_addr: SocketAddr = format!("{host}:{port}").parse()?;
    info!("🌿 papermintd daemon listening on http://{bind_addr}");

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Creates the Axum application router.
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/api/devices", get(devices_handler))
        .route("/api/print", post(print_handler))
        .route("/api/status", post(status_handler))
        .route("/api/drawer", post(drawer_handler))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

// ── Models ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_secs: u64,
}

#[derive(Debug, Serialize)]
pub struct DevicesResponse {
    pub serial_ports: Vec<String>,
    pub usb_printers: Vec<UsbDeviceInfo>,
}

#[derive(Debug, Serialize)]
pub struct UsbDeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
    pub product_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TargetConfig {
    Tcp { address: String },
    Serial { port: String, baud: Option<u32> },
    Usb { vendor_id: u16, product_id: u16 },
    Mock,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DialectChoice {
    #[default]
    EscPos,
    Star,
}

#[derive(Debug, Deserialize)]
pub struct TicketItem {
    pub qty: String,
    #[serde(alias = "name")]
    pub description: String,
    pub price: Option<String>,
    pub total: String,
}

#[derive(Debug, Deserialize)]
pub struct TicketMetadata {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct TicketPayload {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub address: Option<String>,
    pub metadata: Option<Vec<TicketMetadata>>,
    pub items: Option<Vec<TicketItem>>,
    pub totals: Option<Vec<TicketMetadata>>,
    pub qr: Option<String>,
    pub barcode: Option<String>,
    pub footer: Option<String>,
    pub cut_mode: Option<String>, // "full", "partial", "none"
    pub open_drawer: Option<bool>,
    pub beep: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub struct PrintRequest {
    pub target: TargetConfig,
    #[serde(default)]
    pub dialect: DialectChoice,
    #[serde(default = "default_paper_width")]
    pub paper_width: String,
    #[serde(default)]
    pub ticket: Option<TicketPayload>,
    #[serde(default)]
    pub raw_bytes: Option<Vec<u8>>,
}

fn default_paper_width() -> String {
    "80mm".to_string()
}

#[derive(Debug, Serialize)]
pub struct PrintResponse {
    pub success: bool,
    pub bytes_sent: usize,
    pub mock_preview: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StatusRequest {
    pub target: TargetConfig,
    #[serde(default)]
    pub dialect: DialectChoice,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub is_online: bool,
    pub is_ready: bool,
    pub paper: String,
    pub cover: String,
    pub drawer: String,
    pub cutter_error: bool,
    pub head_overheated: bool,
}

// ── Handlers ───────────────────────────────────────────────────────────

async fn health_handler(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: state.start_time.elapsed().as_secs(),
    })
}

async fn devices_handler() -> Json<DevicesResponse> {
    #[cfg(feature = "serial")]
    let serial_ports = SerialTransport::available_ports().unwrap_or_default();
    #[cfg(not(feature = "serial"))]
    let serial_ports = Vec::new();

    #[cfg(feature = "usb")]
    let usb_printers = UsbTransport::list_printers()
        .unwrap_or_default()
        .into_iter()
        .map(|p| UsbDeviceInfo {
            vendor_id: p.vendor_id,
            product_id: p.product_id,
            serial_number: p.serial_number,
            manufacturer: p.manufacturer,
            product_name: p.product_name,
        })
        .collect();
    #[cfg(not(feature = "usb"))]
    let usb_printers = Vec::new();

    Json(DevicesResponse {
        serial_ports,
        usb_printers,
    })
}

async fn print_handler(
    Json(req): Json<PrintRequest>,
) -> Result<Json<PrintResponse>, (StatusCode, String)> {
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
                            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                    } else {
                        printer
                            .print(&receipt)
                            .await
                            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
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
                            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                    } else {
                        printer
                            .print(&receipt)
                            .await
                            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                    }
                    printer.transport().bytes()
                }
            };

            let preview = String::from_utf8_lossy(&bytes)
                .chars()
                .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
                .collect();

            Ok(Json(PrintResponse {
                success: true,
                bytes_sent: bytes.len(),
                mock_preview: Some(preview),
            }))
        }
        TargetConfig::Tcp { address } => {
            let sock: SocketAddr = address.parse().map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    format!("invalid TCP socket address: {e}"),
                )
            })?;
            let transport = papermint::TcpTransport::new(sock)
                .with_connect_timeout(Duration::from_secs(3))
                .with_write_timeout(Duration::from_secs(5));

            match req.dialect {
                DialectChoice::Star => {
                    let mut printer = Printer::new(Star::new(), transport);
                    printer
                        .print(&receipt)
                        .await
                        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
                }
                DialectChoice::EscPos => {
                    let mut printer = Printer::new(EscPos::new(), transport);
                    printer
                        .print(&receipt)
                        .await
                        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
                }
            }

            Ok(Json(PrintResponse {
                success: true,
                bytes_sent: 0,
                mock_preview: None,
            }))
        }
        TargetConfig::Serial { port, baud } => {
            #[cfg(feature = "serial")]
            {
                let b = baud.unwrap_or(19200);
                let transport = SerialTransport::new(&port, b);
                match req.dialect {
                    DialectChoice::Star => {
                        let mut printer = Printer::new(Star::new(), transport);
                        printer
                            .print(&receipt)
                            .await
                            .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
                    }
                    DialectChoice::EscPos => {
                        let mut printer = Printer::new(EscPos::new(), transport);
                        printer
                            .print(&receipt)
                            .await
                            .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
                    }
                }
                Ok(Json(PrintResponse {
                    success: true,
                    bytes_sent: 0,
                    mock_preview: None,
                }))
            }
            #[cfg(not(feature = "serial"))]
            {
                let _ = (port, baud);
                Err((
                    StatusCode::NOT_IMPLEMENTED,
                    "Serial feature disabled".into(),
                ))
            }
        }
        TargetConfig::Usb {
            vendor_id,
            product_id,
        } => {
            #[cfg(feature = "usb")]
            {
                let transport = UsbTransport::from_vid_pid(vendor_id, product_id);
                match req.dialect {
                    DialectChoice::Star => {
                        let mut printer = Printer::new(Star::new(), transport);
                        printer
                            .print(&receipt)
                            .await
                            .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
                    }
                    DialectChoice::EscPos => {
                        let mut printer = Printer::new(EscPos::new(), transport);
                        printer
                            .print(&receipt)
                            .await
                            .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
                    }
                }
                Ok(Json(PrintResponse {
                    success: true,
                    bytes_sent: 0,
                    mock_preview: None,
                }))
            }
            #[cfg(not(feature = "usb"))]
            {
                let _ = (vendor_id, product_id);
                Err((StatusCode::NOT_IMPLEMENTED, "USB feature disabled".into()))
            }
        }
    }
}

async fn status_handler(
    Json(req): Json<StatusRequest>,
) -> Result<Json<StatusResponse>, (StatusCode, String)> {
    match req.target {
        TargetConfig::Mock => {
            let status = match req.dialect {
                DialectChoice::Star => {
                    let mut printer = Printer::star_mock();
                    printer.transport().set_read_response(vec![0x12]);
                    printer
                        .query_status()
                        .await
                        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                }
                DialectChoice::EscPos => {
                    let mut printer = Printer::escpos_mock();
                    printer.transport().set_read_response(vec![0x12]);
                    printer
                        .query_status()
                        .await
                        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                }
            };

            Ok(Json(map_status(status)))
        }
        TargetConfig::Tcp { address } => {
            let sock: SocketAddr = address.parse().map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    format!("invalid TCP socket address: {e}"),
                )
            })?;
            let transport = papermint::TcpTransport::new(sock)
                .with_connect_timeout(Duration::from_secs(3))
                .with_read_timeout(Duration::from_secs(3));

            let status = match req.dialect {
                DialectChoice::Star => {
                    let mut printer = Printer::new(Star::new(), transport);
                    printer
                        .query_status()
                        .await
                        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?
                }
                DialectChoice::EscPos => {
                    let mut printer = Printer::new(EscPos::new(), transport);
                    printer
                        .query_status()
                        .await
                        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?
                }
            };
            Ok(Json(map_status(status)))
        }
        _ => Err((
            StatusCode::NOT_IMPLEMENTED,
            "Real-time status query currently only supported over TCP and Mock".into(),
        )),
    }
}

async fn drawer_handler(
    Json(req): Json<StatusRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let print_req = PrintRequest {
        target: req.target,
        dialect: req.dialect,
        paper_width: "80mm".to_string(),
        ticket: Some(TicketPayload {
            open_drawer: Some(true),
            cut_mode: Some("none".to_string()),
            ..Default::default()
        }),
        raw_bytes: None,
    };
    let _ = print_handler(Json(print_req)).await?;

    Ok((StatusCode::OK, "Cash drawer pulse triggered"))
}

fn build_receipt_from_ticket(paper_width: PaperWidth, ticket: TicketPayload) -> Receipt {
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

    if let Some(meta) = ticket.metadata {
        receipt = receipt.divider('-');
        for m in meta {
            receipt = receipt.two_column(m.label, m.value);
        }
    }

    if let Some(items) = ticket.items {
        receipt = receipt.divider('=');
        let columns = [
            TableColumn::fixed(4, Alignment::Left),
            TableColumn::fraction(0.52, Alignment::Left),
            TableColumn::fraction(0.20, Alignment::Right),
            TableColumn::fraction(0.24, Alignment::Right),
        ];
        receipt = receipt.table_header(&["QTY", "ITEM", "PRICE", "TOTAL"], &columns);
        receipt = receipt.divider('-');
        for item in items {
            let unit = item.price.unwrap_or_default();
            receipt = receipt.row(&[&item.qty, &item.description, &unit, &item.total]);
        }
    }

    if let Some(totals) = ticket.totals {
        receipt = receipt.divider('=');
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

fn map_status(s: papermint::PrinterStatus) -> StatusResponse {
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn test_health_endpoint() {
        let state = Arc::new(AppState {
            start_time: Instant::now(),
        });
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_print_mock_ticket() {
        let state = Arc::new(AppState {
            start_time: Instant::now(),
        });
        let app = create_router(state);

        let payload = serde_json::json!({
            "target": { "type": "mock" },
            "dialect": "escpos",
            "paper_width": "80mm",
            "ticket": {
                "title": "MINT BISTRO",
                "items": [
                    { "qty": "1x", "name": "Truffle Burger", "price": "$12.00", "total": "$12.00" }
                ],
                "totals": [
                    { "label": "TOTAL:", "value": "$12.00" }
                ],
                "qr": "https://example.com",
                "cut_mode": "full"
            }
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/print")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
