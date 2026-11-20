use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::settings::Settings;
use crate::storage::Store;

#[derive(Debug, Default)]
pub struct Suppression {
    until: Option<Instant>,
}

impl Suppression {
    pub fn arm(&mut self, window: Duration) {
        self.until = Some(Instant::now() + window);
    }

    pub fn should_skip(&mut self) -> bool {
        match self.until {
            // Clipboard formats can be re-encoded by the OS (for example images),
            // so treat the whole short window as our own write.
            Some(until) if Instant::now() <= until => {
                self.until = None;
                true
            }
            Some(_) => {
                self.until = None;
                false
            }
            None => false,
        }
    }
}

pub struct AppState {
    pub store: Arc<Mutex<Store>>,
    pub settings: Arc<Mutex<Settings>>,
    pub config_path: PathBuf,
    pub suppress: Arc<Mutex<Suppression>>,
    pub media_dir: PathBuf,
    pub transfer_ids: Mutex<Vec<String>>,
    pub target_hwnd: Mutex<Option<isize>>,
}
