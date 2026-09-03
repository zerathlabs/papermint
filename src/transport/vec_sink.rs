//! In-memory transport sink for testing and offline generation.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::error::Result;
use crate::transport::Transport;

/// An in-memory [`Transport`] that records all written bytes.
///
/// Useful for unit tests, offline rasterization, and debugging command sequences.
#[derive(Debug, Clone, Default)]
pub struct VecSink {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl VecSink {
    /// Creates a new empty [`VecSink`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Returns a copy of all bytes written so far.
    #[must_use]
    pub fn bytes(&self) -> Vec<u8> {
        self.buffer.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Clears the recorded bytes buffer.
    pub fn clear(&self) {
        self.buffer.lock().unwrap_or_else(|e| e.into_inner()).clear();
    }

    /// Returns the total number of bytes written so far.
    #[must_use]
    pub fn len(&self) -> usize {
        self.buffer.lock().unwrap_or_else(|e| e.into_inner()).len()
    }

    /// Returns whether any bytes have been written.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[async_trait]
impl Transport for VecSink {
    async fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.buffer
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .extend_from_slice(buf);
        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}
