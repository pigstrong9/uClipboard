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

pub struct PreparedItem {
    pub item: NewItem,
}

pub fn prepare(capture: RawCapture, media_dir: &Path, settings: &Settings) -> Option<PreparedItem> {
    match capture.kind {
        ClipKind::Files => {
            let mut files = capture.files;
            if files.is_empty() {
                return None;
            }
            files.sort();
            let first = files.first()?.as_str();
            let first_name = Path::new(first)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("文件")
                .to_string();
            let preview = format!("{first_name} 等 {} 个文件", files.len());
            let content = serde_json::to_string(&files).ok()?;
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
            let extra = capture
                .html
                .filter(|html| !html.is_empty())
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
    let live = match store.lock() {
        Ok(store) => {
            let _ = store.prune(settings.history_limit, settings.retention_days);
            store.image_paths().unwrap_or_default()
        }
        Err(_) => return,
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

pub fn spawn_monitor(
    app: AppHandle,
    store: Arc<Mutex<Store>>,
    settings: Arc<Mutex<Settings>>,
    suppress: Arc<Mutex<Suppression>>,
    media_dir: PathBuf,
) {
    thread::spawn(move || {
        let source = SystemClipboard;
        let mut last_hash: Option<String> = None;
        let mut last_sequence: Option<u32> = None;
        let mut last_prune = Instant::now();
        let mut was_paused = false;
        let mut resume_sequence: Option<u32> = None;
        let mut skip_next = false;

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
                Err(_) => continue,
            };
            if snapshot.monitor_paused {
                was_paused = true;
                continue;
            }
            if was_paused {
                was_paused = false;
                resume_sequence = platform::clipboard_sequence();
                skip_next = resume_sequence.is_none();
            }

            // Skip read/decode entirely while the clipboard is unchanged; otherwise
            // an image sitting in the clipboard would be re-encoded every poll.
            let sequence = platform::clipboard_sequence();
            if let Some(sequence) = sequence {
                if last_sequence == Some(sequence) {
                    continue;
                }
                last_sequence = Some(sequence);
            }

            let Some(capture) = source.read() else {
                continue;
            };
            let Some(hash) = raw_hash(&capture) else {
                continue;
            };

            if let Some(sequence) = resume_sequence {
                if platform::clipboard_sequence() == Some(sequence) {
                    resume_sequence = None;
                    last_hash = Some(hash);
                    continue;
                }
                resume_sequence = None;
            }
            if last_hash.as_deref() == Some(hash.as_str()) {
                continue;
            }
            if skip_next {
                skip_next = false;
                last_hash = Some(hash);
                continue;
            }

            if source.is_private() || !snapshot.allows(capture.kind) {
                last_hash = Some(hash);
                continue;
            }

            let foreground = platform::foreground_app_name();
            if let Some(app) = foreground.as_deref() {
                if snapshot.ignores_app(app) {
                    last_hash = Some(hash);
                    continue;
                }
            }

            let Some(mut prepared) = prepare(capture, &media_dir, &snapshot) else {
                last_hash = Some(hash);
                continue;
            };
            prepared.item.source_app = foreground;
            let hash = prepared.item.hash.clone();

            {
                let mut guard = match suppress.lock() {
                    Ok(guard) => guard,
                    Err(_) => continue,
                };
                if guard.should_skip() {
                    last_hash = Some(hash);
                    continue;
                }
            }

            let result = store.lock().map(|store| store.insert(prepared.item));
            match result {
                Ok(Ok(_)) => {
                    last_hash = Some(hash);
                    let _ = app.emit("clipboard-changed", ());
                    prune_after_insert(&store, &media_dir, &snapshot);
                    last_prune = Instant::now();
                }
                Ok(Err(error)) => eprintln!("clipboard insert failed: {error}"),
                Err(_) => eprintln!("clipboard store lock failed"),
            }
        }
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
