use std::path::Path;

use clipboard_rs::common::RustImage;
use clipboard_rs::{Clipboard, ClipboardContext, RustImageData};

use crate::models::{ClipItem, ClipKind};

pub struct RawCapture {
    pub kind: ClipKind,
    pub text: Option<String>,
    pub html: Option<String>,
    pub image_png: Option<Vec<u8>>,
    pub files: Vec<String>,
}

pub trait ClipboardSource: Send + Sync {
    fn read(&self) -> Option<RawCapture>;
    fn write(&self, item: &ClipItem, media_dir: &Path) -> bool;
}

pub struct SystemClipboard;

impl ClipboardSource for SystemClipboard {
    fn read(&self) -> Option<RawCapture> {
        let ctx = ClipboardContext::new().ok()?;

        if let Ok(files) = ctx.get_files() {
            if !files.is_empty() {
                return Some(RawCapture {
                    kind: ClipKind::Files,
                    text: None,
                    html: None,
                    image_png: None,
                    files,
                });
            }
        }

        if let Ok(image) = ctx.get_image() {
            if !image.is_empty() {
                if let Ok(png) = image.to_png() {
                    return Some(RawCapture {
                        kind: ClipKind::Image,
                        text: None,
                        html: None,
                        image_png: Some(png.get_bytes().to_vec()),
                        files: Vec::new(),
                    });
                }
            }
        }

        if let Ok(text) = ctx.get_text() {
            if !text.trim().is_empty() {
                let html = ctx.get_html().ok().filter(|h| !h.trim().is_empty());
                return Some(RawCapture {
                    kind: ClipKind::Text,
                    text: Some(text),
                    html,
                    image_png: None,
                    files: Vec::new(),
                });
            }
        }

        None
    }

    fn write(&self, item: &ClipItem, media_dir: &Path) -> bool {
        let Ok(ctx) = ClipboardContext::new() else {
            return false;
        };
        match item.kind {
            ClipKind::Text => ctx.set_text(item.content.clone()).is_ok(),
            ClipKind::Files => {
                let Ok(files) = serde_json::from_str::<Vec<String>>(&item.content) else {
                    return false;
                };
                ctx.set_files(files).is_ok()
            }
            ClipKind::Image => {
                let path = media_dir.join(&item.content);
                let Ok(bytes) = std::fs::read(path) else {
                    return false;
                };
                let Ok(image) = RustImageData::from_bytes(&bytes) else {
                    return false;
                };
                ctx.set_image(image).is_ok()
            }
        }
    }
}
