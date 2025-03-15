use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct ChannelWriter {
    tx: mpsc::Sender<String>,
    buffer: Arc<Mutex<String>>,
}

impl ChannelWriter {
    pub fn new(tx: mpsc::Sender<String>) -> Self {
        Self {
            tx,
            buffer: Arc::new(Mutex::new(String::new())),
        }
    }
    
    fn flush_buffer(&self) -> io::Result<()> {
        let mut buffer = self.buffer.lock().map_err(|_| {
            io::Error::new(io::ErrorKind::Other, "Failed to lock buffer")
        })?;
        
        if !buffer.is_empty() {
            let output = std::mem::take(&mut *buffer);
            // Use try_send to avoid blocking
            let _ = self.tx.try_send(output);
        }
        
        Ok(())
    }
}

impl Write for ChannelWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if let Ok(s) = std::str::from_utf8(buf) {
            let mut buffer = self.buffer.lock().map_err(|_| {
                io::Error::new(io::ErrorKind::Other, "Failed to lock buffer")
            })?;
            
            buffer.push_str(s);
            
            // If we have a newline, flush the buffer
            if s.contains('\n') {
                drop(buffer); // Release lock before flush
                self.flush_buffer()?;
            }
        }
        
        Ok(buf.len())
    }
    
    fn flush(&mut self) -> io::Result<()> {
        self.flush_buffer()
    }
}

// Helper to redirect stdout to a channel
pub async fn with_captured_output<F, Fut, T>(f: F) -> (T, Vec<String>)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = T>,
{
    let (tx, mut rx) = mpsc::channel::<String>(100);
    let writer = ChannelWriter::new(tx.clone());
    let writer_clone = writer.clone();
    
    // TODO: Implement actual stdout redirection if needed
    
    let result = f().await;
    drop(tx);
    
    let mut outputs = Vec::new();
    while let Some(line) = rx.recv().await {
        outputs.push(line);
    }
    
    (result, outputs)
}
