//! Axum HTTP REST API router and request handlers for papermintd.

use std::sync::Arc;

use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use axum::routing::{get, post};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

#[cfg(feature = "serial")]
use papermint::SerialTransport;
#[cfg(feature = "usb")]
use papermint::UsbTransport;

use crate::executor::{execute_print, execute_status};
use crate::models::{
    AppState, DevicesResponse, HealthResponse, PrintRequest, PrintResponse, StatusRequest,
    StatusResponse, TicketPayload, UsbDeviceInfo,
};

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
    execute_print(req)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

async fn status_handler(
    Json(req): Json<StatusRequest>,
) -> Result<Json<StatusResponse>, (StatusCode, String)> {
    execute_status(req)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
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
    let _ = execute_print(print_req)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok((StatusCode::OK, "Cash drawer pulse triggered"))
}
