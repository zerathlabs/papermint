//! Example showing how to format and send a receipt to a network or mock printer.
//!
//! Run this example:
//!   `cargo run --example print_demo`

use papermint::{PaperWidth, Printer, Receipt};
use std::net::SocketAddr;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌿 papermint — Demo Receipt Printer");

    let receipt = Receipt::new(PaperWidth::Mm80)
        .init()
        .center()
        .bold(true)
        .double_size(true)
        .text_ln("MINT BISTRO")
        .double_size(false)
        .bold(false)
        .text_ln("123 Main Street")
        .text_ln("Tel: +1 555 0199")
        .divider('-')
        .left()
        .two_column("1x Truffle Burger", "3.50")
        .two_column("1x Truffle Fries", "1.75")
        .two_column("1x Mint Cooler", "1.25")
        .divider('=')
        .bold(true)
        .two_column("SUBTOTAL", "6.50")
        .two_column("TAX", "0.00")
        .two_column("TOTAL", "6.50")
        .bold(false)
        .divider('-')
        .center()
        .qr("https://example.com/order/9912")
        .feed(2)
        .text_ln("Scan QR code to view digital invoice")
        .feed(3)
        .cut_full()
        .open_drawer();

    let target_addr: SocketAddr = "127.0.0.1:9101".parse()?;
    println!("Attempting to print to virtual printer at {target_addr}...");

    let transport = papermint::transport::tcp::TcpTransport::new(target_addr)
        .with_connect_timeout(Duration::from_millis(500))
        .with_write_timeout(Duration::from_millis(500));
    let mut printer = Printer::new(papermint::EscPos::new(), transport);

    match printer.print(&receipt).await {
        Ok(()) => {
            println!("✅ Receipt sent successfully to virtual printer!");
        }
        Err(e) => {
            println!("⚠️ Network printer not connected at {target_addr}: {e}");
            println!("\nFallback: Testing via in-memory mock (VecSink)...");
            let mut mock_printer = Printer::escpos_mock();
            mock_printer.print(&receipt).await?;
            let bytes = mock_printer.transport().bytes();
            println!(
                "✅ In-memory mock captured {} bytes successfully!",
                bytes.len()
            );
            println!("Preview:\n{}", String::from_utf8_lossy(&bytes));
        }
    }

    Ok(())
}
