//! RS-232 / Virtual COM port serial transport for thermal printers.

use std::time::Duration;

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;
pub use tokio_serial::{DataBits, FlowControl, Parity, SerialPortBuilderExt, SerialStream, StopBits};

use crate::error::{PapermintError, Result};
use crate::transport::Transport;

/// Asynchronous serial transport for RS-232 and USB-to-Serial virtual COM thermal printers.
///
/// Thermal receipt printers connected over serial require accurate baud rate configuration
/// and typically benefit from hardware flow control (RTS/CTS) to prevent the printer's
/// internal receive buffer (typically 4KB) from overflowing during high-speed printing.
#[derive(Debug)]
pub struct SerialTransport {
    port_path: String,
    baud_rate: u32,
    data_bits: DataBits,
    stop_bits: StopBits,
    parity: Parity,
    flow_control: FlowControl,
    stream: Option<SerialStream>,
    write_timeout: Duration,
    read_timeout: Duration,
}

impl SerialTransport {
    /// Creates a new serial transport targeting the specified port and baud rate with default 8N1 settings.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use papermint::transport::serial::SerialTransport;
    ///
    /// let transport = SerialTransport::new("/dev/ttyUSB0", 19200)
    ///     .flow_control_hardware();
    /// ```
    #[must_use]
    pub fn new(port_path: impl Into<String>, baud_rate: u32) -> Self {
        Self {
            port_path: port_path.into(),
            baud_rate,
            data_bits: DataBits::Eight,
            stop_bits: StopBits::One,
            parity: Parity::None,
            flow_control: FlowControl::None,
            stream: None,
            write_timeout: Duration::from_secs(5),
            read_timeout: Duration::from_secs(5),
        }
    }

    /// Returns the configured port path.
    #[must_use]
    pub fn port_path(&self) -> &str {
        &self.port_path
    }

    /// Returns the configured baud rate.
    #[must_use]
    pub const fn baud_rate(&self) -> u32 {
        self.baud_rate
    }

    /// Returns the configured flow control.
    #[must_use]
    pub const fn flow_control(&self) -> FlowControl {
        self.flow_control
    }

    /// Sets the baud rate.
    #[must_use]
    pub const fn with_baud_rate(mut self, baud_rate: u32) -> Self {
        self.baud_rate = baud_rate;
        self
    }

    /// Configures the flow control mechanism.
    #[must_use]
    pub const fn with_flow_control(mut self, flow_control: FlowControl) -> Self {
        self.flow_control = flow_control;
        self
    }

    /// Convenience builder to enable RTS/CTS hardware flow control.
    #[must_use]
    pub const fn flow_control_hardware(self) -> Self {
        self.with_flow_control(FlowControl::Hardware)
    }

    /// Convenience builder to enable XON/XOFF software flow control.
    #[must_use]
    pub const fn flow_control_software(self) -> Self {
        self.with_flow_control(FlowControl::Software)
    }

    /// Convenience builder to disable flow control.
    #[must_use]
    pub const fn flow_control_none(self) -> Self {
        self.with_flow_control(FlowControl::None)
    }

    /// Configures the character data bits (Seven or Eight).
    #[must_use]
    pub const fn with_data_bits(mut self, data_bits: DataBits) -> Self {
        self.data_bits = data_bits;
        self
    }

    /// Configures the stop bits (One or Two).
    #[must_use]
    pub const fn with_stop_bits(mut self, stop_bits: StopBits) -> Self {
        self.stop_bits = stop_bits;
        self
    }

    /// Configures the parity check (None, Odd, or Even).
    #[must_use]
    pub const fn with_parity(mut self, parity: Parity) -> Self {
        self.parity = parity;
        self
    }

    /// Configures the write timeout.
    #[must_use]
    pub const fn with_write_timeout(mut self, timeout: Duration) -> Self {
        self.write_timeout = timeout;
        self
    }

    /// Configures the read timeout.
    #[must_use]
    pub const fn with_read_timeout(mut self, timeout: Duration) -> Self {
        self.read_timeout = timeout;
        self
    }

    /// Connects to the serial port if not already open.
    pub async fn connect(&mut self) -> Result<()> {
        if self.stream.is_some() {
            return Ok(());
        }

        let builder = tokio_serial::new(&self.port_path, self.baud_rate)
            .data_bits(self.data_bits)
            .stop_bits(self.stop_bits)
            .parity(self.parity)
            .flow_control(self.flow_control);

        let stream = builder.open_native_async().map_err(|e| {
            PapermintError::Serial(format!(
                "failed to open serial port {}: {}",
                self.port_path, e
            ))
        })?;

        self.stream = Some(stream);
        Ok(())
    }

    /// Closes the serial connection.
    pub async fn disconnect(&mut self) -> Result<()> {
        self.stream = None;
        Ok(())
    }
}

#[async_trait]
impl Transport for SerialTransport {
    async fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        if self.stream.is_none() {
            self.connect().await?;
        }

        let stream = self.stream.as_mut().expect("stream must be connected");
        match timeout(self.write_timeout, stream.write_all(buf)).await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(PapermintError::Io(e)),
            Err(_) => Err(PapermintError::Timeout(
                self.write_timeout.as_millis() as u64
            )),
        }
    }

    async fn flush(&mut self) -> Result<()> {
        if let Some(stream) = self.stream.as_mut() {
            match timeout(self.write_timeout, stream.flush()).await {
                Ok(Ok(())) => Ok(()),
                Ok(Err(e)) => Err(PapermintError::Io(e)),
                Err(_) => Err(PapermintError::Timeout(
                    self.write_timeout.as_millis() as u64
                )),
            }
        } else {
            Ok(())
        }
    }

    async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.stream.is_none() {
            self.connect().await?;
        }

        let stream = self.stream.as_mut().expect("stream must be connected");
        match timeout(self.read_timeout, stream.read(buf)).await {
            Ok(Ok(n)) => Ok(n),
            Ok(Err(e)) => Err(PapermintError::Io(e)),
            Err(_) => Err(PapermintError::Timeout(
                self.read_timeout.as_millis() as u64
            )),
        }
    }
}

