//! Example: Restaurant Dining Bill & Guest Check
//!
//! Demonstrates:
//! - 80mm standard paper width (48 columns)
//! - Header branding with centered typography
//! - Stateful N-column itemized bill with automatic word-wrapping
//! - Subtotal, tax breakdown, and tip suggestions
//! - Highlighting total with double-height typography
//! - Contactless payment QR code
//! - Code128 transaction barcode for POS scanner recall
//! - Drawer kick and full auto-cut
//!
//! Run with:
//!   `cargo run --example restaurant_bill`

use papermint::{Alignment, PaperWidth, Printer, Receipt, TableColumn};
use std::net::SocketAddr;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🍽️  papermint — Restaurant Dining Bill & Guest Check Demo\n");

    let columns = [
        TableColumn::fixed(4, Alignment::Left),        // Qty
        TableColumn::fraction(0.50, Alignment::Left),  // Description
        TableColumn::fraction(0.22, Alignment::Right), // Unit Price
        TableColumn::fraction(0.24, Alignment::Right), // Total Price
    ];

    // Build the restaurant guest check receipt
    let receipt = Receipt::new(PaperWidth::Mm80)
        .init()
        // Header
        .center()
        .bold(true)
        .double_size(true)
        .text_ln("THE MINT BISTRO")
        .double_size(false)
        .bold(false)
        .text_ln("Fine Dining & Wood-Fired Kitchen")
        .text_ln("742 Evergreen Terrace, Suite 100")
        .text_ln("Tel: (555) 019-2834 • GSTIN: 29ABCDE1234F1Z5")
        .divider('=')
        // Metadata
        .left()
        .two_column("Table: 14 (Dine-In)", "Guests: 3")
        .two_column("Server: Alex M.", "Order: #1042")
        .two_column("Date: 2026-09-09", "Time: 19:45 EST")
        .divider('-')
        // Itemized Table using stateful builder
        .table_header(&["QTY", "DESCRIPTION", "PRICE", "TOTAL"], &columns)
        .divider('-')
        .row(&[
            "2x",
            "Truffle Wagyu Smash Burger w/ Aged Cheddar & Onion Jam",
            "$14.50",
            "$29.00",
        ])
        .row(&[
            "1x",
            "Wood-Fired Margherita Pizza (Fresh Mozzarella & Basil)",
            "$18.00",
            "$18.00",
        ])
        .row(&[
            "2x",
            "San Pellegrino Blood Orange Mint Cooler",
            "$4.50",
            "$9.00",
        ])
        .row(&[
            "1x",
            "Artisanal Affogato al Caffe w/ Double Espresso",
            "$6.50",
            "$6.50",
        ])
        .divider('-')
        // Financials
        .two_column("Subtotal:", "$62.50")
        .two_column("State Sales Tax (6.25%):", "$3.91")
        .two_column("City Hospitality Tax (2.00%):", "$1.25")
        .divider('=')
        .bold(true)
        .double_height(true)
        .two_column("BALANCE DUE:", "$67.66")
        .double_height(false)
        .bold(false)
        .feed(1)
        // Suggested Gratuity Guide
        .center()
        .text_ln("--- Suggested Gratuity Guide ---")
        .two_column("15%: $9.38  |  18%: $11.25  |  20%:", "$12.50")
        .feed(1)
        // Payment info
        .divider('-')
        .left()
        .two_column("Payment: VISA Contactless", "Card: **** 4242")
        .two_column("Auth: 088921", "Ref: TXN-839201")
        .divider('-')
        // Contactless QR & Digital Bill
        .center()
        .text_ln("Scan to pay online or split bill:")
        .feed(1)
        .qr("https://pay.mintbistro.com/bill/1042")
        .feed(1)
        // POS Barcode
        .barcode_128("TX-1042-839201")
        .feed(1)
        .text_ln("Thank you for joining us tonight!")
        .feed(3)
        .cut_full()
        .open_drawer();

    println!("Receipt commands count: {}", receipt.commands().len());

    // Try sending to local virtual printer or fallback to in-memory mock
    let target_addr: SocketAddr = "127.0.0.1:9101".parse()?;
    let transport = papermint::transport::tcp::TcpTransport::new(target_addr)
        .with_connect_timeout(Duration::from_millis(300))
        .with_write_timeout(Duration::from_millis(300));
    let mut printer = Printer::new(papermint::EscPos::new(), transport);

    match printer.print(&receipt).await {
        Ok(()) => {
            println!("✅ Receipt transmitted directly to printer at {target_addr}!");
        }
        Err(_) => {
            // Emulate and preview via in-memory mock printer
            let mut mock_printer = Printer::escpos_mock();
            mock_printer.print(&receipt).await?;
            let bytes = mock_printer.transport().bytes();
            println!("⚡ Wire bytes encoded: {} bytes", bytes.len());
            println!("\n--- [TERMINAL RECEIPT PREVIEW (ESC/POS)] ---");
            // Strip ESC control codes for readable terminal preview
            let text_preview = String::from_utf8_lossy(&bytes);
            let cleaned: String = text_preview
                .chars()
                .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
                .collect();
            println!("{cleaned}");
            println!("--- [END PREVIEW] ---");
        }
    }

    Ok(())
}
