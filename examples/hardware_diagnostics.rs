//! Example: Hardware Diagnostics & Device Discovery CLI
//!
//! Demonstrates:
//! - Enumerating system serial / COM ports (`SerialTransport::available_ports()`)
//! - Enumerating attached USB Printer Class devices (`UsbTransport::list_printers()`)
//! - Querying real-time printer hardware telemetry (`query_status()`)
//! - Evaluating sensor flags: Paper level, Cover open, Drawer kick, Cutter jam, Head overheat
//! - Human-readable health diagnostic reporting
//!
//! Run with:
//!   `cargo run --example hardware_diagnostics --all-features`

use papermint::{CoverStatus, DrawerStatus, PaperStatus, Printer, PrinterStatus};
#[cfg(feature = "serial")]
use papermint::SerialTransport;
#[cfg(feature = "usb")]
use papermint::UsbTransport;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 papermint — Hardware Diagnostics & Device Discovery\n");

    // 1. Scan Serial / COM Ports
    println!("══════════════════════════════════════════════════");
    println!(" 1. Serial / COM Port Enumeration");
    println!("══════════════════════════════════════════════════");
    #[cfg(feature = "serial")]
    {
        match SerialTransport::available_ports() {
            Ok(ports) if ports.is_empty() => {
                println!("  [i] No hardware serial ports detected on host system.");
            }
            Ok(ports) => {
                println!("  Found {} serial port(s):", ports.len());
                for (idx, port) in ports.iter().enumerate() {
                    println!("   {}. {}", idx + 1, port);
                }
            }
            Err(e) => println!("  [!] Failed to enumerate serial ports: {e}"),
        }
    }
    #[cfg(not(feature = "serial"))]
    {
        println!("  [i] Serial feature disabled in compilation.");
    }

    // 2. Scan USB Printer Class Devices
    println!("\n══════════════════════════════════════════════════");
    println!(" 2. USB Printer Class (0x07) Enumeration");
    println!("══════════════════════════════════════════════════");
    #[cfg(feature = "usb")]
    {
        match UsbTransport::list_printers() {
            Ok(printers) if printers.is_empty() => {
                println!("  [i] No USB Printer Class (0x07) devices found on host.");
                println!("      (If running in WSL, attach USB via `usbipd wsl attach`)");
            }
            Ok(printers) => {
                println!("  Found {} USB printer(s):", printers.len());
                for (idx, p) in printers.iter().enumerate() {
                    let mfg = p.manufacturer.as_deref().unwrap_or("Unknown");
                    let prod = p.product_name.as_deref().unwrap_or("Generic Printer");
                    let serial = p.serial_number.as_deref().unwrap_or("N/A");
                    println!(
                        "   {}. VID: 0x{:04X} | PID: 0x{:04X} | {} - {} (S/N: {})",
                        idx + 1,
                        p.vendor_id,
                        p.product_id,
                        mfg,
                        prod,
                        serial
                    );
                }
            }
            Err(e) => println!("  [!] Failed to enumerate USB devices: {e}"),
        }
    }
    #[cfg(not(feature = "usb"))]
    {
        println!("  [i] USB feature disabled in compilation.");
    }

    // 3. Real-Time Hardware Status Telemetry Diagnostic Report
    println!("\n══════════════════════════════════════════════════");
    println!(" 3. Real-Time Status Telemetry Simulation");
    println!("══════════════════════════════════════════════════");

    // We simulate querying an active ESC/POS printer that responds with real-time status bytes
    // Wire byte 0x12 represents: Ready, Paper OK, Cover Closed, Drawer Closed, Normal Temp
    let ready_response = [0x12];
    let mut printer = Printer::escpos_mock();
    printer.transport().set_read_response(ready_response.to_vec());

    let status = printer.query_status().await?;
    print_diagnostic_report("PRINTER-01 (Primary Counter)", &status);

    // Now simulate an issue scenario: Paper Near-End + Cash Drawer Open
    // ESC/POS DLE EOT 1 status byte: bit 2 (0x04) = drawer open, bit 5/6 = paper near-end
    let warning_response = [0x12 | 0x04 | 0x20];
    printer.transport().set_read_response(warning_response.to_vec());

    let warning_status = printer.query_status().await?;
    print_diagnostic_report("PRINTER-02 (Drive-Thru)", &warning_status);

    Ok(())
}

fn print_diagnostic_report(printer_name: &str, status: &PrinterStatus) {
    println!("\n--- Health Report: {printer_name} ---");

    let overall_badge = if status.is_ready() {
        "🟢 [ONLINE / READY]"
    } else {
        "🔴 [ATTENTION REQUIRED]"
    };
    println!("Overall Status:    {overall_badge}");

    let paper_info = match status.paper {
        PaperStatus::Adequate => "🟢 Adequate paper loaded",
        PaperStatus::NearEnd => "🟡 WARNING: Paper roll is low (near-end)",
        PaperStatus::Empty => "🔴 ERROR: Paper roll is empty",
    };
    println!("Paper Level:       {paper_info}");

    let cover_info = match status.cover {
        CoverStatus::Closed => "🟢 Cover is securely closed",
        CoverStatus::Open => "🔴 ERROR: Cover is currently open",
    };
    println!("Casing / Cover:    {cover_info}");

    let drawer_info = match status.drawer {
        DrawerStatus::Closed => "⚪ Drawer is closed",
        DrawerStatus::Open => "🔵 Drawer is open",
    };
    println!("Cash Drawer:       {drawer_info}");

    let cutter_info = if status.cutter_error {
        "🔴 ERROR: Auto-cutter jammed!"
    } else {
        "🟢 Auto-cutter mechanism normal"
    };
    println!("Cutter Assembly:   {cutter_info}");

    let head_info = if status.head_overheated {
        "🔴 ERROR: Thermal printhead overheated! Pausing."
    } else {
        "🟢 Printhead temperature within normal operating limits"
    };
    println!("Head Temperature:  {head_info}");
}
