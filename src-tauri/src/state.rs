use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::settings::Settings;
use crate::storage::Store;

/// Remembers the clipboard write we just performed so the monitor can ignore it.
///
/// The old implementation skipped the next change unconditionally for a fixed window,
/// which also discarded a real copy made right after a paste. Matching the write itself
/// (payload hash, plus the clipboard sequence on Windows) keeps that from happening.
#[derive(Debug, Default)]
pub struct Suppression {
    until: Option<Instant>,
    hash: Option<String>,
    /// Image round-trips can be re-encoded by the OS, so the payload hash may differ on
    /// platforms without a usable change counter. The short time window covers that case.
    allow_mismatch: bool,
    #[cfg(target_os = "windows")]
    sequence: Option<u32>,
}

impl Suppression {
    pub fn arm(&mut self, window: Duration, hash: Option<String>, allow_mismatch: bool) {
        self.until = Some(Instant::now() + window);
        self.hash = hash;
        self.allow_mismatch = allow_mismatch;
        #[cfg(target_os = "windows")]
        {
            // Captured after the write, so it identifies the revision we produced.
            self.sequence = crate::platform::clipboard_sequence();
        }
    }

    /// Returns true when `(sequence, hash)` is the write armed above, not a new copy.
    pub fn should_skip(&mut self, sequence: Option<u32>, hash: &str) -> bool {
        let Some(until) = self.until else {
            return false;
        };
        if Instant::now() > until {
            self.reset();
            return false;
        }
        if self.hash.as_deref() == Some(hash) {
            self.reset();
            return true;
        }
        #[cfg(target_os = "windows")]
        {
            if let Some(expected) = self.sequence {
                // The sequence only moves when someone writes the clipboard. If it is still
                // ours, the content is ours even when the format was re-encoded.
                if sequence == Some(expected) {
                    self.reset();
                    return true;
                }
                // A different sequence means the user copied something new in the meantime.
                self.reset();
                return false;
            }
        }
        if self.allow_mismatch {
            self.reset();
            return true;
        }
        // Nothing identifies this content as our write, so record it normally.
        self.reset();
        false
    }

    fn reset(&mut self) {
        self.until = None;
        self.hash = None;
        self.allow_mismatch = false;
        #[cfg(target_os = "windows")]
        {
            self.sequence = None;
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
    pub screenshot: Mutex<Option<crate::screenshot::ScreenshotSession>>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_only_the_armed_payload() {
        let mut suppression = Suppression::default();
        suppression.arm(Duration::from_secs(5), Some("ours".to_string()), false);
        assert!(suppression.should_skip(None, "ours"));

        suppression.arm(Duration::from_secs(5), Some("ours".to_string()), false);
        assert!(!suppression.should_skip(None, "a real copy"));
    }

    #[test]
    fn expires_after_the_window() {
        let mut suppression = Suppression::default();
        suppression.arm(Duration::ZERO, Some("ours".to_string()), false);
        std::thread::sleep(Duration::from_millis(2));
        assert!(!suppression.should_skip(None, "ours"));
    }
}
