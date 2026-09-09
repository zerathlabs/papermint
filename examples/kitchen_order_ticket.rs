//! Example: Production Kitchen Order Ticket (KOT)
//!
//! Demonstrates:
//! - 80mm high-visibility kitchen station layout
//! - High-contrast inverted order header with large order number
//! - Multi-column item list with double-size item titles
//! - Indented dietary warning flags and allergen callouts
//! - Multi-ticket routing metadata (Table, Station, Expediter, Time)
//! - Double acoustic beeper cue to alert kitchen staff in noisy environments
//! - Partial cut (`cut_partial()`) allowing chefs to tear off the ticket cleanly
//!
//! Run with:
//!   `cargo run --example kitchen_order_ticket`

use papermint::{Alignment, PaperWidth, Printer, Receipt, TableColumn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("👨‍🍳 papermint — High-Urgency Kitchen Order Ticket (KOT) Demo\n");

    let item_columns = [
        TableColumn::fixed(5, Alignment::Left),         // Qty (e.g. "2x  ")
        TableColumn::fraction(1.0, Alignment::Left),   // Item & Special instructions
    ];

    let kot = Receipt::new(PaperWidth::Mm80)
        .init()
        // Urgent Header Banner
        .center()
        .invert(true)
        .bold(true)
        .double_size(true)
        .text_ln("  KITCHEN ORDER #1042  ")
        .double_size(false)
        .invert(false)
        .bold(false)
        .feed(1)
        // Station & Table Information
        .left()
        .bold(true)
        .two_column("STATION: HOT LINE / GRILL", "PRIORITY: RUSH")
        .two_column("TABLE: 14 (DINE-IN)", "COVERS: 3")
        .bold(false)
        .two_column("Server: Alex M.", "Sent: 19:46:12")
        .divider('=')
        // Items & Modifiers
        .set_columns(&item_columns)
        .bold(true)
        .double_height(true)
        .row(&["2x", "TRUFFLE WAGYU BURGER"])
        .double_height(false)
        .bold(false)
        .text_ln("     >> ** NO ONIONS **")
        .text_ln("     >> ** MEDIUM RARE **")
        .text_ln("     >> ADD EXTRA PICKLES")
        .feed(1)
        .bold(true)
        .double_height(true)
        .row(&["1x", "WOOD-FIRED MARGHERITA"])
        .double_height(false)
        .bold(false)
        .text_ln("     >> CRISPY CRUST / WELL DONE")
        .text_ln("     >> FRESH BASIL ON SIDE")
        .feed(1)
        .bold(true)
        .double_height(true)
        .row(&["2x", "SAN PELLEGRINO COOLER"])
        .double_height(false)
        .bold(false)
        .text_ln("     >> EXTRA ICE")
        .feed(1)
        .divider('=')
        // Allergen & Expediter Warning Callout
        .center()
        .bold(true)
        .invert(true)
        .text_ln(" !!! ALLERGY ALERT: GUEST HAS SESAME ALLERGY !!! ")
        .invert(false)
        .bold(false)
        .left()
        .text_ln("Notes: Hold mains until starters are cleared.")
        .divider('-')
        .center()
        .text_ln("[ END OF TICKET #1042 ]")
        .feed(3)
        // Audio alert: 3 loud beeps to notify prep cooks
        .beep(3, 2)
        // Partial cut keeps ticket hanging slightly for easy tear-off
        .cut_partial();

    // Print to in-memory mock and output terminal preview
    let mut mock_printer = Printer::escpos_mock();
    mock_printer.print(&kot).await?;
    let bytes = mock_printer.transport().bytes();

    println!("⚡ Generated {} wire bytes of KOT data.", bytes.len());
    println!("\n--- [TERMINAL KOT PREVIEW] ---");
    let text_preview = String::from_utf8_lossy(&bytes);
    let cleaned: String = text_preview
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .collect();
    println!("{cleaned}");
    println!("--- [END PREVIEW] ---");

    Ok(())
}

