//! Network TCP transport for IP/Ethernet thermal printers.

use std::net::SocketAddr;
use std::time::Duration;

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::error::{PapermintError, Result};
use crate::transport::Transport;

/// Async TCP socket transport for networked thermal receipt printers.
#[derive(Debug)]
pub struct TcpTransport {
    addr: SocketAddr,
    stream: Option<TcpStream>,
    connect_timeout: Duration,
    write_timeout: Duration,
    read_timeout: Duration,
}

impl TcpTransport {
    /// Creates a new TCP transport targeting the given socket address with default timeouts (5s).
    #[must_use]
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            stream: None,
            connect_timeout: Duration::from_secs(5),
            write_timeout: Duration::from_secs(5),
            read_timeout: Duration::from_secs(5),
        }
    }

    /// Configures the connection timeout.
    #[must_use]
    pub const fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Configures the write timeout.
    #[must_use]
    pub const fn with_write_timeout(mut self, timeout: Duration) -> Self {
        self.write_timeout = timeout;
        self
    }

    /// Configures the read timeout for telemetry and responses.
    #[must_use]
    pub const fn with_read_timeout(mut self, timeout: Duration) -> Self {
        self.read_timeout = timeout;
        self
    }

    /// Connects to the printer if not already connected.
    pub async fn connect(&mut self) -> Result<()> {
        if self.stream.is_some() {
            return Ok(());
        }

        let stream = match timeout(self.connect_timeout, TcpStream::connect(self.addr)).await {
            Ok(Ok(stream)) => {
                // Disable Nagle's algorithm so short POS commands (drawer kicks, cuts, line feeds)
                // are transmitted immediately without 40–200ms ACK coalescing latency.
                let _ = stream.set_nodelay(true);
                stream
            }
            Ok(Err(e)) => return Err(PapermintError::Io(e)),
            Err(_) => {
                return Err(PapermintError::Timeout(
                    self.connect_timeout.as_millis() as u64
                ))
            }
        };

        self.stream = Some(stream);
        Ok(())
    }

    /// Closes the connection.
    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(mut stream) = self.stream.take() {
            let _ = stream.shutdown().await;
        }
        Ok(())
    }
}

#[async_trait]
impl Transport for TcpTransport {
    async fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        if self.stream.is_none() {
            self.connect().await?;
        }

        let stream = self.stream.as_mut().expect("stream must be connected");
        match timeout(self.write_timeout, stream.write_all(buf)).await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => {
                // Invalidate stream so that next call attempts reconnection
                self.stream = None;
                Err(PapermintError::Io(e))
            }
            Err(_) => {
                self.stream = None;
                Err(PapermintError::Timeout(
                    self.write_timeout.as_millis() as u64
                ))
            }
        }
    }

    async fn flush(&mut self) -> Result<()> {
        if let Some(stream) = self.stream.as_mut() {
            match timeout(self.write_timeout, stream.flush()).await {
                Ok(Ok(())) => Ok(()),
                Ok(Err(e)) => {
                    self.stream = None;
                    Err(PapermintError::Io(e))
                }
                Err(_) => {
                    self.stream = None;
                    Err(PapermintError::Timeout(
                        self.write_timeout.as_millis() as u64
                    ))
                }
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
            Ok(Ok(0)) => {
                // EOF / connection closed by remote peer
                self.stream = None;
                Ok(0)
            }
            Ok(Ok(n)) => Ok(n),
            Ok(Err(e)) => {
                self.stream = None;
                Err(PapermintError::Io(e))
            }
            Err(_) => {
                self.stream = None;
                Err(PapermintError::Timeout(
                    self.read_timeout.as_millis() as u64,
                ))
            }
        }
    }
}

