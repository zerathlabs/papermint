//! papermintd — Lightweight native HTTP print sidecar daemon and WebSocket cloud gateway.
//!
//! Provides a high-performance local microservice for streaming POS print jobs,
//! checking real-time sensor telemetry, and enumerating hardware devices over HTTP/REST,
//! with optional outbound persistent WebSocket cloud printing gateway.

pub mod cli;
pub mod executor;
pub mod gateway;
pub mod models;
pub mod routes;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use tracing::info;

pub use executor::{execute_print, execute_status};
pub use models::*;
pub use routes::create_router;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "papermintd=info,tower_http=info".into()),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();
    let config = match cli::parse_cli_args(&args)? {
        Some(cfg) => cfg,
        None => return Ok(()), // --help displayed
    };

    if let Some(url) = config.gateway_url {
        let gw_config = gateway::GatewayConfig {
            url,
            token: config.gateway_token,
            shop_id: config.shop_id,
            default_target: config.default_target,
        };

        info!("🚀 Launching WebSocket Cloud Printing Gateway background task...");
        tokio::spawn(async move {
            gateway::run_gateway_loop(gw_config).await;
        });
    }

    let state = Arc::new(AppState {
        start_time: Instant::now(),
    });

    let app = create_router(state);

    let bind_addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    info!("🌿 papermintd daemon listening on http://{bind_addr}");

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
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

    #[test]
    fn test_parse_targets() {
        match parse_target_str("mock").unwrap() {
            TargetConfig::Mock => {}
            _ => panic!("expected Mock"),
        }
        match parse_target_str("tcp://192.168.1.100:9100").unwrap() {
            TargetConfig::Tcp { address } => assert_eq!(address, "192.168.1.100:9100"),
            _ => panic!("expected Tcp"),
        }
        match parse_target_str("usb:04b8:0202").unwrap() {
            TargetConfig::Usb {
                vendor_id,
                product_id,
            } => {
                assert_eq!(vendor_id, 0x04b8);
                assert_eq!(product_id, 0x0202);
            }
            _ => panic!("expected Usb"),
        }
        match parse_target_str("serial:/dev/ttyUSB0:19200").unwrap() {
            TargetConfig::Serial { port, baud } => {
                assert_eq!(port, "/dev/ttyUSB0");
                assert_eq!(baud, Some(19200));
            }
            _ => panic!("expected Serial"),
        }
    }

    #[tokio::test]
    async fn test_execute_print_mock_raw() {
        let req = PrintRequest {
            target: TargetConfig::Mock,
            dialect: DialectChoice::EscPos,
            paper_width: "80mm".into(),
            ticket: None,
            raw_bytes: Some(b"\x1b@Hello Raw World\n".to_vec()),
        };

        let res = execute_print(req).await.unwrap();
        assert!(res.success);
        assert!(res.bytes_sent > 0);
        assert!(res.mock_preview.unwrap().contains("Hello Raw World"));
    }
}
