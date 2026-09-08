//! Printer hardware status and sensor telemetry models.

/// Thermal paper roll sensor status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaperStatus {
    /// Adequate paper remaining on roll.
    #[default]
    Adequate,
    /// Paper roll is running low (near-end sensor triggered).
    NearEnd,
    /// Paper roll is completely empty.
    Empty,
}

/// Printer cover mechanical status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CoverStatus {
    /// Printer cover is closed and latched.
    #[default]
    Closed,
    /// Printer cover is open.
    Open,
}

/// Cash drawer microswitch sensor status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DrawerStatus {
    /// Cash drawer is closed.
    #[default]
    Closed,
    /// Cash drawer is open.
    Open,
}

/// Comprehensive hardware status report decoded from real-time telemetry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PrinterStatus {
    /// Whether the printer is online and ready to accept print jobs.
    pub is_online: bool,
    /// Printer head/cover mechanical status.
    pub cover: CoverStatus,
    /// Thermal paper roll sensor status.
    pub paper: PaperStatus,
    /// Cash drawer microswitch sensor status.
    pub drawer: DrawerStatus,
    /// Auto-cutter mechanical error or paper jam.
    pub cutter_error: bool,
    /// Thermal print head is overheated.
    pub head_overheated: bool,
}

impl PrinterStatus {
    /// Returns `true` if the printer is ready to print without errors or empty paper.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.is_online
            && self.cover == CoverStatus::Closed
            && self.paper != PaperStatus::Empty
            && !self.cutter_error
            && !self.head_overheated
    }
}
