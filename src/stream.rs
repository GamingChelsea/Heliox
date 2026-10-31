use std::{
    fs::File,
    io::{Read, Seek},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

use symphonia::core::io::MediaSource;

pub struct StreamState {
    pub downloaded_bytes: AtomicUsize,
    pub total_bytes: Option<u64>,
    pub is_finished: AtomicBool,
    pub is_cancelled: AtomicBool,
}

pub struct ProgressiveReader {
    pub file: File,
    pub state: Arc<StreamState>,
}
impl ProgressiveReader {
    pub fn new(file: File, state: Arc<StreamState>) -> Self {
        Self { file, state }
    }
}

impl MediaSource for ProgressiveReader {
    fn is_seekable(&self) -> bool {
        self.state.total_bytes.is_some() || self.state.is_finished.load(Ordering::Relaxed)
    }

    fn byte_len(&self) -> Option<u64> {
        self.state.total_bytes.or_else(|| {
            if self.state.is_finished.load(Ordering::Relaxed) {
                Some(self.state.downloaded_bytes.load(Ordering::Relaxed) as u64)
            } else {
                None
            }
        })
    }
}

impl Read for ProgressiveReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            if self.state.is_cancelled.load(Ordering::Relaxed) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    "Stream cancelled",
                ));
            }

            let current_pos = self.file.stream_position()? as usize;
            let downloaded = self.state.downloaded_bytes.load(Ordering::Relaxed);

            if current_pos < downloaded {
                let max_to_read = buf.len().min(downloaded - current_pos);
                return self.file.read(&mut buf[..max_to_read]);
            }

            if self.state.is_finished.load(Ordering::Relaxed) {
                return Ok(0);
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}

impl Seek for ProgressiveReader {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let target = match pos {
            std::io::SeekFrom::End(offset) => {
                if let Some(total) = self.state.total_bytes {
                    let target_pos = (total as i64 + offset).max(0) as u64;
                    std::io::SeekFrom::Start(target_pos)
                } else {
                    pos
                }
            }
            other => other,
        };
        self.file.seek(target)
    }
}
