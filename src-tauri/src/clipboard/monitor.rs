use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde_json::json;
use tauri::{AppHandle, Emitter};

use crate::media;
use crate::models::{ClipKind, NewItem};
use crate::storage::Store;

use super::hash::{hash_bytes, hash_files};
use super::source::{ClipboardSource, RawCapture, SystemClipboard};

const POLL_INTERVAL: Duration = Duration::from_millis(300);
const MAX_TEXT_BYTES: usize = 5 * 1024 * 1024;
const MAX_IMAGE_BYTES: usize = 20 * 1024 * 1024;
const MIN_TEXT_CHARS: usize = 3;

pub struct PreparedItem {
    pub item: NewItem,
}

pub fn prepare(capture: RawCapture, media_dir: &Path) -> Option<PreparedItem> {
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
            if png.is_empty() || png.len() > MAX_IMAGE_BYTES {
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
            if text.chars().count() < MIN_TEXT_CHARS || text.len() > MAX_TEXT_BYTES {
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

pub fn spawn_monitor(
    app: AppHandle,
    store: Arc<Mutex<Store>>,
    suppress: Arc<Mutex<Option<String>>>,
    media_dir: PathBuf,
) {
    thread::spawn(move || {
        let source = SystemClipboard;
        let mut last_hash: Option<String> = None;

        loop {
            thread::sleep(POLL_INTERVAL);

            let Some(capture) = source.read() else {
                continue;
            };
            let Some(prepared) = prepare(capture, &media_dir) else {
                continue;
            };
            let hash = prepared.item.hash.clone();

            if last_hash.as_deref() == Some(hash.as_str()) {
                continue;
            }

            {
                let mut guard = suppress.lock().unwrap();
                if guard.as_deref() == Some(hash.as_str()) {
                    *guard = None;
                    last_hash = Some(hash);
                    continue;
                }
            }

            let result = store.lock().unwrap().insert(prepared.item);
            match result {
                Ok(_) => {
                    last_hash = Some(hash);
                    let _ = app.emit("clipboard-changed", ());
                }
                Err(error) => eprintln!("clipboard insert failed: {error}"),
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
        let prepared = prepare(text_capture("ab", None), Path::new("."));
        assert!(prepared.is_none());
    }

    #[test]
    fn prepares_text_preview_and_html() {
        let prepared = prepare(
            text_capture("hello   clipboard world", Some("<b>hello</b>")),
            Path::new("."),
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
                files: vec![
                    "C:/tmp/b.txt".to_string(),
                    "C:/tmp/a.txt".to_string(),
                ],
            },
            Path::new("."),
        )
        .expect("files should be prepared");
        assert_eq!(prepared.item.kind, ClipKind::Files);
        assert_eq!(prepared.item.size, 2);
        assert_eq!(prepared.item.content, "[\"C:/tmp/a.txt\",\"C:/tmp/b.txt\"]");
        assert_eq!(prepared.item.preview, "a.txt 等 2 个文件");
    }
}
