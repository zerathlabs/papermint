//! Command-line argument parsing and environment variable configuration for papermintd.

use crate::models::{TargetConfig, parse_target_str};

/// Application runtime configuration parsed from CLI flags and environment variables.
#[derive(Debug, Clone)]
pub struct CliConfig {
    pub host: String,
    pub port: u16,
    pub gateway_url: Option<String>,
    pub gateway_token: Option<String>,
    pub shop_id: Option<String>,
    pub default_target: Option<TargetConfig>,
}

/// Parses CLI arguments, falling back to environment variables and defaults.
pub fn parse_cli_args(args: &[String]) -> Result<Option<CliConfig>, String> {
    let mut i = 1;
    let mut gateway_url = std::env::var("GATEWAY_URL")
        .or_else(|_| std::env::var("PAPERMINTD_GATEWAY_URL"))
        .ok();
    let mut gateway_token = std::env::var("GATEWAY_TOKEN")
        .or_else(|_| std::env::var("PAPERMINTD_GATEWAY_TOKEN"))
        .ok();
    let mut shop_id = std::env::var("SHOP_ID")
        .or_else(|_| std::env::var("PAPERMINTD_SHOP_ID"))
        .ok();
    let mut default_printer_str = std::env::var("DEFAULT_PRINTER")
        .or_else(|_| std::env::var("PAPERMINTD_DEFAULT_PRINTER"))
        .ok();
    let mut host = std::env::var("HOST")
        .or_else(|_| std::env::var("PAPERMINTD_HOST"))
        .unwrap_or_else(|_| "127.0.0.1".to_string());
    let mut port: u16 = std::env::var("PORT")
        .or_else(|_| std::env::var("PAPERMINTD_PORT"))
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    while i < args.len() {
        match args[i].as_str() {
            "-g" | "--gateway" => {
                i += 1;
                if let Some(val) = args.get(i) {
                    gateway_url = Some(val.clone());
                }
            }
            "-t" | "--token" => {
                i += 1;
                if let Some(val) = args.get(i) {
                    gateway_token = Some(val.clone());
                }
            }
            "-s" | "--shop-id" => {
                i += 1;
                if let Some(val) = args.get(i) {
                    shop_id = Some(val.clone());
                }
            }
            "-p" | "--printer" => {
                i += 1;
                if let Some(val) = args.get(i) {
                    default_printer_str = Some(val.clone());
                }
            }
            "--port" => {
                i += 1;
                if let Some(p) = args.get(i).and_then(|s| s.parse().ok()) {
                    port = p;
                }
            }
            "--host" => {
                i += 1;
                if let Some(val) = args.get(i) {
                    host = val.clone();
                }
            }
            "-h" | "--help" => {
                print_help();
                return Ok(None);
            }
            _ => {}
        }
        i += 1;
    }

    let default_target = match default_printer_str {
        Some(ref s) => Some(parse_target_str(s).map_err(|e| format!("Invalid --printer: {e}"))?),
        None => None,
    };

    Ok(Some(CliConfig {
        host,
        port,
        gateway_url,
        gateway_token,
        shop_id,
        default_target,
    }))
}

pub fn print_help() {
    println!(
        r#"papermintd v{}
High-performance thermal printing sidecar daemon and WebSocket cloud gateway.

USAGE:
    papermintd [OPTIONS]

OPTIONS:
    -h, --help               Print this help message
    --host <HOST>            HTTP server bind address (default: 127.0.0.1, env: HOST)
    --port <PORT>            HTTP server port (default: 8080, env: PORT)
    -g, --gateway <WSS_URL>  Outbound WebSocket URL to cloud server (env: GATEWAY_URL)
    -t, --token <TOKEN>      Authentication bearer token for cloud gateway (env: GATEWAY_TOKEN)
    -s, --shop-id <ID>       Unique shop identifier (env: SHOP_ID)
    -p, --printer <TARGET>   Default printer target (e.g. 'mock', 'tcp://192.168.1.100:9100', 'usb:04b8:0202')
"#,
        env!("CARGO_PKG_VERSION")
    );
}
