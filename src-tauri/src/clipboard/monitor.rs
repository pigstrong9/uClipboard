use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::json;
use tauri::{AppHandle, Emitter};

use crate::media;
use crate::models::{ClipKind, NewItem};
use crate::platform;
use crate::settings::Settings;
use crate::state::Suppression;
use crate::storage::Store;

use super::hash::{hash_bytes, hash_files};
use super::source::{ClipboardSource, RawCapture, SystemClipboard};

const POLL_INTERVAL: Duration = Duration::from_millis(300);
const PRUNE_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);
/// A single capture should never be able to fill the database or the IPC payload.
const MAX_FILES: usize = 200;
const MAX_FILES_JSON_BYTES: usize = 2 * 1024 * 1024;
const MAX_HTML_BYTES: usize = 512 * 1024;

pub struct PreparedItem {
    pub item: NewItem,
}

pub fn prepare(capture: RawCapture, media_dir: &Path, settings: &Settings) -> Option<PreparedItem> {
    match capture.kind {
        ClipKind::Files => {
            let mut files: Vec<String> = capture
                .files
                .into_iter()
                .filter(|path| !path.is_empty())
                .collect();
            files.sort();
            files.dedup();
            if files.is_empty() {
                return None;
            }
            files.truncate(MAX_FILES);
            let mut content = serde_json::to_string(&files).ok()?;
            // Paths can be arbitrarily long, so shrink the batch until the JSON fits.
            while content.len() > MAX_FILES_JSON_BYTES && files.len() > 1 {
                files.truncate(files.len() / 2);
                content = serde_json::to_string(&files).ok()?;
            }
            if content.len() > MAX_FILES_JSON_BYTES {
                return None;
            }
            let first = files.first()?.as_str();
            let first_name = Path::new(first)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("文件")
                .to_string();
            let preview = format!("{first_name} 等 {} 个文件", files.len());
            Some(PreparedItem {
                item: NewItem {
                    kind: ClipKind::Files,
                    content,
                    extra: Some(json!({ "count": files.len() }).to_string()),
                    preview,
                    hash: hash_files(&files),
                    size: files.len() as i64,
                    source_app: None,
                },
            })
        }
        ClipKind::Image => {
            let png = capture.image_png?;
            if png.is_empty() || png.len() > settings.max_image_bytes {
                return None;
            }
            let hash = hash_bytes(&png);
            let saved = media::save_png(media_dir, &hash, &png).ok()?;
            Some(PreparedItem {
                item: NewItem {
                    kind: ClipKind::Image,
                    content: saved.rel_path,
                    extra: Some(
                        json!({ "width": saved.width, "height": saved.height }).to_string(),
                    ),
                    preview: String::new(),
                    hash,
                    size: png.len() as i64,
                    source_app: None,
                },
            })
        }
        ClipKind::Text => {
            let text = capture.text?;
            // When enabled, trim the plain text before it is validated, hashed or
            // stored, so history never keeps a copy of the surrounding whitespace.
            let text = if settings.trim_text {
                text.trim().to_string()
            } else {
                text
            };
            if text.chars().count() < settings.min_text_length
                || text.len() > settings.max_text_bytes
            {
                return None;
            }
            let preview = text
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .chars()
                .take(120)
                .collect::<String>();
            // HTML is only a convenience for rich paste; an oversized document is dropped
            // rather than stored next to every text clip.
            let extra = capture
                .html
                .filter(|html| !html.is_empty() && html.len() <= MAX_HTML_BYTES)
                .map(|html| json!({ "html": html }).to_string());
            Some(PreparedItem {
                item: NewItem {
                    kind: ClipKind::Text,
                    content: text.clone(),
                    extra,
                    preview,
                    hash: hash_bytes(text.as_bytes()),
                    size: text.chars().count() as i64,
                    source_app: None,
                },
            })
        }
    }
}

fn advance_sequence(last: &mut Option<u32>, sequence: Option<u32>) {
    if let Some(sequence) = sequence {
        *last = Some(sequence);
    }
}

fn raw_hash(capture: &RawCapture) -> Option<String> {
    match capture.kind {
        ClipKind::Text => capture
            .text
            .as_ref()
            .map(|text| hash_bytes(text.as_bytes())),
        ClipKind::Image => capture.image_png.as_ref().map(|png| hash_bytes(png)),
        ClipKind::Files => {
            if capture.files.is_empty() {
                None
            } else {
                Some(hash_files(&capture.files))
            }
        }
    }
}

fn prune_and_cleanup(store: &Arc<Mutex<Store>>, media_dir: &Path, settings: &Settings) {
    let live = {
        let Ok(store) = store.lock() else {
            return;
        };
        let _ = store.prune(settings.history_limit, settings.retention_days);
        match store.image_paths() {
            Ok(live) => live,
            Err(error) => {
                log::warn!("media cleanup skipped: {error}");
                return;
            }
        }
    };
    media::cleanup_orphans(media_dir, &live);
}

fn prune_after_insert(store: &Arc<Mutex<Store>>, media_dir: &Path, settings: &Settings) {
    let Ok(store) = store.lock() else {
        return;
    };
    if let Ok(images) = store.prune(settings.history_limit, settings.retention_days) {
        media::remove_media_files(media_dir, &images);
    }
}

fn report_error(app: &AppHandle, last: &mut Option<String>, message: String) {
    if last.as_deref() == Some(message.as_str()) {
        return;
    }
    log::error!("clipboard monitor: {message}");
    let _ = app.emit("monitor-error", message.clone());
    *last = Some(message);
}

fn clear_error(app: &AppHandle, last: &mut Option<String>) {
    if last.take().is_some() {
        let _ = app.emit("monitor-ok", ());
    }
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "unknown panic payload".to_string()
    }
}

fn monitor_loop(
    app: AppHandle,
    store: Arc<Mutex<Store>>,
    settings: Arc<Mutex<Settings>>,
    suppress: Arc<Mutex<Suppression>>,
    media_dir: PathBuf,
) {
    let source = SystemClipboard;
    let mut last_hash: Option<String> = None;
    let mut last_sequence: Option<u32> = None;
    let mut last_prune = Instant::now();
    let mut was_paused = false;
    let mut last_error: Option<String> = None;

    loop {
        thread::sleep(POLL_INTERVAL);

        if last_prune.elapsed() >= PRUNE_INTERVAL {
            let snapshot = settings
                .lock()
                .map(|settings| settings.clone())
                .unwrap_or_default();
            prune_and_cleanup(&store, &media_dir, &snapshot);
            last_prune = Instant::now();
        }

        let snapshot = match settings.lock() {
            Ok(settings) => settings.clone(),
            Err(_) => {
                report_error(&app, &mut last_error, "monitor.settings_failed".to_string());
                continue;
            }
        };
        if snapshot.monitor_paused {
            was_paused = true;
            // Track the revision while paused so content copied during the pause is
            // skipped on resume. Recording it as the last sequence (instead of
            // snapshotting the content on resume) means a copy made just after
            // resuming is not mistaken for paused content.
            advance_sequence(&mut last_sequence, platform::clipboard_sequence());
            continue;
        }
        if was_paused {
            was_paused = false;
        }

        // Skip read/decode entirely while the clipboard is unchanged; otherwise
        // an image sitting in the clipboard would be re-encoded every poll.
        let sequence = platform::clipboard_sequence();
        if let Some(sequence) = sequence {
            if last_sequence == Some(sequence) {
                continue;
            }
        }

        // Privacy and the ignore list are checked before reading, so concealed content
        // is never pulled into memory or decoded as an image first.
        let owner_names = platform::clipboard_owner_app_names(&app);
        if owner_names.iter().any(|name| snapshot.ignores_app(name)) {
            advance_sequence(&mut last_sequence, sequence);
            clear_error(&app, &mut last_error);
            continue;
        }

        match source.privacy() {
            Ok(true) => {
                advance_sequence(&mut last_sequence, sequence);
                clear_error(&app, &mut last_error);
                continue;
            }
            Ok(false) => {}
            Err(error) => {
                // A failed format probe may be a transient clipboard lock; retry on the
                // next poll instead of guessing whether the content is private.
                log::error!("privacy check failed: {error}");
                report_error(&app, &mut last_error, "monitor.privacy_failed".to_string());
                continue;
            }
        }

        // A failed read must not advance the revision, or the content would stay
        // invisible until the clipboard changes again.
        let capture = match source.read() {
            Ok(Some(capture)) => capture,
            Ok(None) => {
                advance_sequence(&mut last_sequence, sequence);
                clear_error(&app, &mut last_error);
                continue;
            }
            Err(error) => {
                log::error!("read clipboard failed: {error}");
                report_error(&app, &mut last_error, "monitor.read_failed".to_string());
                continue;
            }
        };
        clear_error(&app, &mut last_error);

        let Some(hash) = raw_hash(&capture) else {
            advance_sequence(&mut last_sequence, sequence);
            continue;
        };
        advance_sequence(&mut last_sequence, sequence);

        {
            let mut guard = match suppress.lock() {
                Ok(guard) => guard,
                Err(_) => {
                    report_error(&app, &mut last_error, "monitor.lock_failed".to_string());
                    continue;
                }
            };
            if guard.should_skip(sequence, &hash) {
                last_hash = Some(hash);
                continue;
            }
        }

        if last_hash.as_deref() == Some(hash.as_str()) {
            continue;
        }

        if !snapshot.allows(capture.kind) {
            last_hash = Some(hash);
            continue;
        }

        let Some(mut prepared) = prepare(capture, &media_dir, &snapshot) else {
            last_hash = Some(hash);
            continue;
        };
        prepared.item.source_app = owner_names.first().cloned();
        let hash = prepared.item.hash.clone();

        let result = store.lock().map(|store| store.insert(prepared.item));
        match result {
            Ok(Ok(_)) => {
                last_hash = Some(hash);
                let _ = app.emit("clipboard-changed", ());
                prune_after_insert(&store, &media_dir, &snapshot);
                last_prune = Instant::now();
            }
            Ok(Err(error)) => {
                log::error!("insert clipboard item failed: {error}");
                report_error(&app, &mut last_error, "monitor.insert_failed".to_string());
            }
            Err(_) => {
                report_error(&app, &mut last_error, "monitor.lock_failed".to_string());
            }
        }
    }
}

pub fn spawn_monitor(
    app: AppHandle,
    store: Arc<Mutex<Store>>,
    settings: Arc<Mutex<Settings>>,
    suppress: Arc<Mutex<Suppression>>,
    media_dir: PathBuf,
) {
    thread::spawn(move || loop {
        // A panic in a decoder or FFI call must not silently stop capture; the thread
        // reports it and starts a fresh loop instead.
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            monitor_loop(
                app.clone(),
                Arc::clone(&store),
                Arc::clone(&settings),
                Arc::clone(&suppress),
                media_dir.clone(),
            );
        }));
        let code = match outcome {
            Ok(()) => {
                log::warn!("clipboard monitor exited unexpectedly");
                "monitor.stopped".to_string()
            }
            Err(payload) => {
                log::error!("clipboard monitor panicked: {}", panic_message(payload));
                "monitor.panicked".to_string()
            }
        };
        let _ = app.emit("monitor-error", code);
        thread::sleep(Duration::from_secs(2));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_capture(text: &str, html: Option<&str>) -> RawCapture {
        RawCapture {
            kind: ClipKind::Text,
            text: Some(text.to_string()),
            html: html.map(str::to_string),
            image_png: None,
            files: Vec::new(),
        }
    }

    #[test]
    fn skips_short_text() {
        let prepared = prepare(
            text_capture("ab", None),
            Path::new("."),
            &Settings::default(),
        );
        assert!(prepared.is_none());
    }

    #[test]
    fn prepares_text_preview_and_html() {
        let prepared = prepare(
            text_capture("hello   clipboard world", Some("<b>hello</b>")),
            Path::new("."),
            &Settings::default(),
        )
        .expect("text should be prepared");
        assert_eq!(prepared.item.kind, ClipKind::Text);
        assert_eq!(prepared.item.preview, "hello clipboard world");
        assert!(prepared
            .item
            .extra
            .as_deref()
            .unwrap_or_default()
            .contains("<b>hello</b>"));
    }

    #[test]
    fn trims_text_when_enabled() {
        let settings = Settings {
            trim_text: true,
            ..Settings::default()
        };
        let prepared = prepare(
            text_capture("  hello clipboard  ", None),
            Path::new("."),
            &settings,
        )
        .expect("text should be prepared");
        assert_eq!(prepared.item.content, "hello clipboard");
        assert_eq!(prepared.item.preview, "hello clipboard");
        assert_eq!(prepared.item.size, 15);
    }

    #[test]
    fn keeps_surrounding_whitespace_when_trim_disabled() {
        let prepared = prepare(
            text_capture("  hello clipboard  ", None),
            Path::new("."),
            &Settings::default(),
        )
        .expect("text should be prepared");
        assert_eq!(prepared.item.content, "  hello clipboard  ");
    }

    #[test]
    fn drops_text_that_becomes_too_short_after_trim() {
        let settings = Settings {
            trim_text: true,
            min_text_length: 3,
            ..Settings::default()
        };
        assert!(prepare(text_capture("   ab   ", None), Path::new("."), &settings).is_none());
    }

    #[test]
    fn drops_oversized_html() {
        let html = "x".repeat(MAX_HTML_BYTES + 1);
        let prepared = prepare(
            text_capture("hello clipboard", Some(&html)),
            Path::new("."),
            &Settings::default(),
        )
        .expect("text should still be prepared");
        assert!(prepared.item.extra.is_none());
    }

    #[test]
    fn caps_the_file_count() {
        let files = (0..(MAX_FILES + 50))
            .map(|index| format!("C:/tmp/{index}.txt"))
            .collect();
        let prepared = prepare(
            RawCapture {
                kind: ClipKind::Files,
                text: None,
                html: None,
                image_png: None,
                files,
            },
            Path::new("."),
            &Settings::default(),
        )
        .expect("files should be prepared");
        assert_eq!(prepared.item.size, MAX_FILES as i64);
    }

    #[test]
    fn prepares_files_sorted() {
        let prepared = prepare(
            RawCapture {
                kind: ClipKind::Files,
                text: None,
                html: None,
                image_png: None,
                files: vec!["C:/tmp/b.txt".to_string(), "C:/tmp/a.txt".to_string()],
            },
            Path::new("."),
            &Settings::default(),
        )
        .expect("files should be prepared");
        assert_eq!(prepared.item.kind, ClipKind::Files);
        assert_eq!(prepared.item.size, 2);
        assert_eq!(prepared.item.content, "[\"C:/tmp/a.txt\",\"C:/tmp/b.txt\"]");
        assert_eq!(prepared.item.preview, "a.txt 等 2 个文件");
    }

    #[test]
    fn respects_record_switches() {
        let settings = Settings {
            record_text: false,
            ..Settings::default()
        };
        assert!(!settings.allows(ClipKind::Text));
        assert!(settings.allows(ClipKind::Image));
    }
}
