//! Example: Kitchen Order Ticket (KOT)
//!
//! Demonstrates:
//! - 58mm compact paper width (32 columns)
//! - Upside-down printing mode (`.upside_down(true)`) for ticket clips above prep stations
//! - Custom line spacing
//! - Inverted header banner (white on black)
//! - Audio buzzer alert on order arrival
//!
//! Run with:
//!   `cargo run --example kitchen_ticket`

use papermint::{PaperWidth, Printer, Receipt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🍳 papermint — Kitchen Order Ticket Demo");

    // Build the kitchen ticket
    let ticket = Receipt::new(PaperWidth::Mm58)
        .init()
        // Enable upside-down printing so cooks can read the ticket while it hangs from a clip rail
        .upside_down(true)
        .center()
        .invert(true)
        .bold(true)
        .text_ln(" ORDER #42 — DINE IN ")
        .invert(false)
        .bold(false)
        .text_ln("Table 7 • Server: Alex")
        .divider('-')
        .left()
        .bold(true)
        .two_column("2x Mint Smash Burger", "")
        .bold(false)
        .text_ln("   - NO ONIONS")
        .text_ln("   - EXTRA PICKLES")
        .feed(1)
        .bold(true)
        .two_column("1x Truffle Fries", "")
        .two_column("2x Mint Cooler", "")
        .bold(false)
        .divider('=')
        .center()
        .text_ln("Time: 12:45 PM")
        .feed(3)
        // Buzzer alert: beep twice to notify kitchen staff
        .beep(2, 2)
        .cut_full();

    // Print to in-memory mock to verify generated wire bytes
    let mut printer = Printer::escpos_mock();
    printer.print(&ticket).await?;

    let bytes = printer.transport().bytes();
    println!("✅ Generated {} bytes of kitchen ticket data.", bytes.len());
    println!(
        "\nPreview of rendered text:\n{}",
        String::from_utf8_lossy(&bytes)
    );

    Ok(())
}
