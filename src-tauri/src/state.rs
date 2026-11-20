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

/// The subset of [`AppState`] that commands hand to blocking tasks. `State<'_, AppState>`
/// is borrowed, so anything wanting to run on a `spawn_blocking` thread clones this instead.
#[derive(Clone)]
pub struct SharedState {
    pub store: Arc<Mutex<Store>>,
    pub settings: Arc<Mutex<Settings>>,
    pub suppress: Arc<Mutex<Suppression>>,
    pub media_dir: PathBuf,
}

impl AppState {
    pub fn shared(&self) -> SharedState {
        SharedState {
            store: Arc::clone(&self.store),
            settings: Arc::clone(&self.settings),
            suppress: Arc::clone(&self.suppress),
            media_dir: self.media_dir.clone(),
        }
    }
}
