//! Outbound WebSocket Cloud Printing Gateway for papermintd.
//!
//! Connects behind NAT/firewalls to a central cloud server (e.g. Next.js, Hono, Node.js)
//! and receives remote print jobs in real-time.

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info, warn};

use crate::{
    DialectChoice, PrintRequest, StatusRequest, StatusResponse, TargetConfig, TicketPayload,
};

#[derive(Debug, Clone)]
pub struct GatewayConfig {
    pub url: String,
    pub token: Option<String>,
    pub shop_id: Option<String>,
    pub default_target: Option<TargetConfig>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CloudMessage {
    PrintJob {
        job_id: String,
        target: Option<TargetConfig>,
        #[serde(default)]
        dialect: DialectChoice,
        #[serde(default = "crate::default_paper_width")]
        paper_width: String,
        #[serde(default)]
        ticket: Option<TicketPayload>,
        #[serde(default)]
        raw_bytes: Option<Vec<u8>>,
    },
    Ping {
        #[serde(default)]
        timestamp: Option<u64>,
    },
    StatusQuery {
        target: Option<TargetConfig>,
        #[serde(default)]
        dialect: DialectChoice,
    },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GatewayMessage {
    Register {
        version: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        shop_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        token: Option<String>,
        capabilities: Vec<String>,
    },
    Ack {
        job_id: String,
        success: bool,
        bytes_sent: usize,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mock_preview: Option<String>,
    },
    Pong {
        timestamp: u64,
    },
    Status {
        status: StatusResponse,
    },
}

/// Runs the persistent WebSocket gateway loop with exponential backoff.
pub async fn run_gateway_loop(config: GatewayConfig) {
    let mut backoff = Duration::from_secs(1);
    let max_backoff = Duration::from_secs(30);

    loop {
        info!(
            "🔌 Connecting to WebSocket printing gateway at {}...",
            config.url
        );
        match connect_and_listen(&config).await {
            Ok(()) => {
                info!("Gateway connection closed cleanly. Reconnecting in 1s...");
                backoff = Duration::from_secs(1);
            }
            Err(err) => {
                warn!("⚠️ Gateway connection error: {err}. Retrying in {backoff:?}...");
                tokio::time::sleep(backoff).await;
                backoff = (backoff * 2).min(max_backoff);
                continue;
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn connect_and_listen(
    config: &GatewayConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (ws_stream, response) = connect_async(&config.url).await?;
    info!(
        "✅ Connected to cloud gateway (HTTP status: {})",
        response.status()
    );

    let (mut write, mut read) = ws_stream.split();

    // Send initial Register handshake
    let register = GatewayMessage::Register {
        version: env!("CARGO_PKG_VERSION").to_string(),
        shop_id: config.shop_id.clone(),
        token: config.token.clone(),
        capabilities: vec![
            "escpos".into(),
            "star".into(),
            "tspl".into(),
            "zpl".into(),
            "raw".into(),
        ],
    };

    let reg_json = serde_json::to_string(&register)?;
    write.send(Message::text(reg_json)).await?;
    info!(
        "📤 Sent gateway registration frame (shop: {:?})",
        config.shop_id
    );

    while let Some(msg_res) = read.next().await {
        let msg = msg_res?;
        match msg {
            Message::Text(text) => {
                let parsed: Result<CloudMessage, _> = serde_json::from_str(&text);
                match parsed {
                    Ok(CloudMessage::PrintJob {
                        job_id,
                        target,
                        dialect,
                        paper_width,
                        ticket,
                        raw_bytes,
                    }) => {
                        let effective_target = target
                            .or_else(|| config.default_target.clone())
                            .unwrap_or(TargetConfig::Mock);

                        let print_req = PrintRequest {
                            target: effective_target,
                            dialect,
                            paper_width,
                            ticket,
                            raw_bytes,
                        };

                        info!("🖨️ Processing cloud print job {job_id}...");
                        let ack = match crate::execute_print(print_req).await {
                            Ok(resp) => {
                                info!(
                                    " Job {job_id} printed successfully ({} bytes sent)",
                                    resp.bytes_sent
                                );
                                GatewayMessage::Ack {
                                    job_id,
                                    success: true,
                                    bytes_sent: resp.bytes_sent,
                                    error: None,
                                    mock_preview: resp.mock_preview,
                                }
                            }
                            Err(err_msg) => {
                                error!("❌ Job {job_id} print failed: {err_msg}");
                                GatewayMessage::Ack {
                                    job_id,
                                    success: false,
                                    bytes_sent: 0,
                                    error: Some(err_msg),
                                    mock_preview: None,
                                }
                            }
                        };

                        let ack_json = serde_json::to_string(&ack)?;
                        write.send(Message::text(ack_json)).await?;
                    }
                    Ok(CloudMessage::Ping { timestamp }) => {
                        let ts = timestamp.unwrap_or_else(|| {
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs()
                        });
                        let pong = GatewayMessage::Pong { timestamp: ts };
                        write
                            .send(Message::text(serde_json::to_string(&pong)?))
                            .await?;
                    }
                    Ok(CloudMessage::StatusQuery { target, dialect }) => {
                        let effective_target = target
                            .or_else(|| config.default_target.clone())
                            .unwrap_or(TargetConfig::Mock);

                        let status_req = StatusRequest {
                            target: effective_target,
                            dialect,
                        };

                        let status_res = match crate::execute_status(status_req).await {
                            Ok(status) => GatewayMessage::Status { status },
                            Err(e) => GatewayMessage::Ack {
                                job_id: "status".into(),
                                success: false,
                                bytes_sent: 0,
                                error: Some(e),
                                mock_preview: None,
                            },
                        };
                        write
                            .send(Message::text(serde_json::to_string(&status_res)?))
                            .await?;
                    }
                    Err(err) => {
                        warn!("⚠️ Unrecognized or malformed cloud message: {err}");
                    }
                }
            }
            Message::Ping(data) => {
                write.send(Message::Pong(data)).await?;
            }
            Message::Close(_) => {
                info!("🚪 Cloud gateway requested connection close");
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;
    use tokio_tungstenite::accept_async;

    #[tokio::test]
    async fn test_gateway_roundtrip() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_addr = listener.local_addr().unwrap();

        // Spawn mock cloud WebSocket server
        let server_handle = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut ws = accept_async(stream).await.unwrap();

            // Receive Register handshake
            let reg_msg = ws.next().await.unwrap().unwrap();
            let reg_text = reg_msg.to_text().unwrap();
            let reg: GatewayMessage = serde_json::from_str(reg_text).unwrap();
            match reg {
                GatewayMessage::Register {
                    shop_id,
                    capabilities,
                    ..
                } => {
                    assert_eq!(shop_id.as_deref(), Some("shop_test_123"));
                    assert!(capabilities.contains(&"escpos".to_string()));
                }
                _ => panic!("expected Register frame"),
            }

            // Dispatch print job to daemon
            let job = CloudMessage::PrintJob {
                job_id: "test_job_999".into(),
                target: Some(TargetConfig::Mock),
                dialect: DialectChoice::EscPos,
                paper_width: "80mm".into(),
                ticket: Some(TicketPayload {
                    title: Some("CLOUD COFFEE".into()),
                    ..Default::default()
                }),
                raw_bytes: None,
            };
            ws.send(Message::text(serde_json::to_string(&job).unwrap()))
                .await
                .unwrap();

            // Verify execution acknowledgement from daemon
            let ack_msg = ws.next().await.unwrap().unwrap();
            let ack_text = ack_msg.to_text().unwrap();
            let ack: GatewayMessage = serde_json::from_str(ack_text).unwrap();
            match ack {
                GatewayMessage::Ack {
                    job_id,
                    success,
                    bytes_sent,
                    mock_preview,
                    ..
                } => {
                    assert_eq!(job_id, "test_job_999");
                    assert!(success);
                    assert!(bytes_sent > 0);
                    assert!(mock_preview.unwrap().contains("CLOUD COFFEE"));
                }
                _ => panic!("expected Ack frame"),
            }

            // Send ping keepalive
            let ping = CloudMessage::Ping {
                timestamp: Some(123456789),
            };
            ws.send(Message::text(serde_json::to_string(&ping).unwrap()))
                .await
                .unwrap();

            // Verify pong reply
            let pong_msg = ws.next().await.unwrap().unwrap();
            let pong_text = pong_msg.to_text().unwrap();
            let pong: GatewayMessage = serde_json::from_str(pong_text).unwrap();
            match pong {
                GatewayMessage::Pong { timestamp } => {
                    assert_eq!(timestamp, 123456789);
                }
                _ => panic!("expected Pong frame"),
            }

            // Gracefully terminate session
            ws.send(Message::Close(None)).await.unwrap();
        });

        // Run client gateway connecting to local mock cloud server
        let config = GatewayConfig {
            url: format!("ws://{local_addr}"),
            token: Some("test_token_xyz".into()),
            shop_id: Some("shop_test_123".into()),
            default_target: Some(TargetConfig::Mock),
        };

        let res = connect_and_listen(&config).await;
        assert!(res.is_ok());

        server_handle.await.unwrap();
    }
}
