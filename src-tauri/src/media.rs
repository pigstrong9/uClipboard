use std::collections::HashSet;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use image::codecs::png::{CompressionType, FilterType as PngFilterType, PngEncoder};
use image::{ExtendedColorType, GenericImageView, ImageEncoder, ImageFormat, Rgba, RgbaImage};

pub struct SavedImage {
    pub rel_path: String,
    pub width: u32,
    pub height: u32,
}

/// Largest edge allowed when decoding an image from disk or the clipboard.
const MAX_DECODE_EDGE: u32 = 20_000;
/// Upper bound on decoded pixels, so a small compressed file cannot expand unbounded.
const MAX_DECODE_PIXELS: u64 = 100_000_000;
/// Merged images are capped separately: a single paste should not build a huge canvas.
const MAX_MERGE_IMAGES: usize = 32;
const MAX_MERGE_PIXELS: u64 = 80_000_000;

/// Decodes an image while enforcing dimension and allocation limits.
pub fn decode_image_limited(bytes: &[u8]) -> Result<image::DynamicImage, String> {
    let mut reader = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|error| error.to_string())?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_DECODE_EDGE);
    limits.max_image_height = Some(MAX_DECODE_EDGE);
    limits.max_alloc = Some(384 * 1024 * 1024);
    reader.limits(limits);
    let image = reader.decode().map_err(|error| error.to_string())?;
    let pixels = u64::from(image.width()) * u64::from(image.height());
    if pixels > MAX_DECODE_PIXELS {
        return Err("image.too_large".to_string());
    }
    Ok(image)
}

pub fn save_png(media_dir: &Path, hash: &str, png: &[u8]) -> Result<SavedImage, String> {
    // Decode before touching the disk so an over-limit image leaves no stray file.
    let img = decode_image_limited(png)?;
    let images = media_dir.join("images");
    std::fs::create_dir_all(&images).map_err(|e| e.to_string())?;
    let path = images.join(format!("{hash}.png"));
    std::fs::write(&path, png).map_err(|e| e.to_string())?;

    let (width, height) = img.dimensions();
    let thumb = img.thumbnail(300, 300).to_rgb8();
    let mut buffer = Cursor::new(Vec::new());
    if thumb.write_to(&mut buffer, ImageFormat::Jpeg).is_ok() {
        let _ = std::fs::write(
            images.join(format!("{hash}_thumb.jpg")),
            buffer.into_inner(),
        );
    }

    Ok(SavedImage {
        rel_path: format!("images/{hash}.png"),
        width,
        height,
    })
}

pub fn thumb_path(media_dir: &Path, rel_path: &str) -> Option<String> {
    let stem = Path::new(rel_path).file_stem()?.to_string_lossy();
    let path = media_dir.join("images").join(format!("{stem}_thumb.jpg"));
    path.is_file().then(|| path.to_string_lossy().into_owned())
}

pub fn remove_media_files(media_dir: &Path, rel_paths: &[String]) {
    let images = media_dir.join("images");
    for rel in rel_paths {
        let Some(stem) = Path::new(rel).file_stem() else {
            continue;
        };
        let stem = stem.to_string_lossy();
        let _ = std::fs::remove_file(images.join(format!("{stem}.png")));
        let _ = std::fs::remove_file(images.join(format!("{stem}_thumb.jpg")));
    }
}

/// Deletes unreferenced files under `images/`. An empty `live_paths` deletes them all, so
/// callers must pass a list the database actually returned, never a failed-read fallback.
pub fn cleanup_orphans(media_dir: &Path, live_paths: &[String]) {
    let images = media_dir.join("images");
    let Ok(entries) = std::fs::read_dir(&images) else {
        return;
    };
    let keep: HashSet<String> = live_paths
        .iter()
        .filter_map(|path| Path::new(path).file_stem())
        .map(|stem| stem.to_string_lossy().to_string())
        .collect();

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let stem = name
            .strip_suffix("_thumb.jpg")
            .or_else(|| name.strip_suffix(".png"))
            .or_else(|| name.strip_suffix(".jpg"));
        if let Some(stem) = stem {
            if !keep.contains(stem) {
                let _ = std::fs::remove_file(path);
            }
        }
    }
}

pub fn merge_images_vertical(paths: &[PathBuf]) -> Result<Vec<u8>, String> {
    if paths.len() > MAX_MERGE_IMAGES {
        return Err("image.merge_too_many".to_string());
    }
    let mut images = Vec::new();
    let mut decoded_pixels = 0u64;
    for path in paths {
        let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
        let image = decode_image_limited(&bytes)?;
        decoded_pixels =
            decoded_pixels.saturating_add(u64::from(image.width()) * u64::from(image.height()));
        if decoded_pixels > MAX_MERGE_PIXELS {
            return Err("image.merge_too_large".to_string());
        }
        images.push(image.to_rgba8());
    }
    if images.is_empty() {
        return Err("image.merge_empty".to_string());
    }

    const GAP: u32 = 8;
    let width = images.iter().map(|image| image.width()).max().unwrap_or(1);
    let height = images
        .iter()
        .map(|image| image.height())
        .sum::<u32>()
        .saturating_add(GAP.saturating_mul(images.len().saturating_sub(1) as u32));
    if width == 0 || height == 0 {
        return Err("image.invalid_size".to_string());
    }
    if u64::from(width) * u64::from(height) > MAX_MERGE_PIXELS {
        return Err("image.merge_too_large".to_string());
    }

    let mut canvas = RgbaImage::from_pixel(width, height, Rgba([255, 255, 255, 255]));
    let mut y = 0i64;
    for image in images {
        let x = ((width - image.width()) / 2) as i64;
        image::imageops::overlay(&mut canvas, &image, x, y);
        y += image.height() as i64 + GAP as i64;
    }

    let mut buffer = Vec::new();
    PngEncoder::new_with_quality(&mut buffer, CompressionType::Fast, PngFilterType::NoFilter)
        .write_image(canvas.as_raw(), width, height, ExtendedColorType::Rgba8)
        .map_err(|error| error.to_string())?;
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("uclipboard-media-{name}-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(dir.join("images")).unwrap();
        dir
    }

    #[test]
    fn cleanup_removes_orphan_images_and_thumbnails() {
        let dir = temp_dir("cleanup");
        let images = dir.join("images");
        std::fs::write(images.join("keep.png"), b"keep").unwrap();
        std::fs::write(images.join("keep_thumb.jpg"), b"thumb").unwrap();
        std::fs::write(images.join("orphan.png"), b"orphan").unwrap();
        std::fs::write(images.join("orphan_thumb.jpg"), b"orphan").unwrap();

        cleanup_orphans(&dir, &["images/keep.png".to_string()]);

        assert!(images.join("keep.png").exists());
        assert!(images.join("keep_thumb.jpg").exists());
        assert!(!images.join("orphan.png").exists());
        assert!(!images.join("orphan_thumb.jpg").exists());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn merges_images_vertically_with_gap() {
        let dir = temp_dir("merge");
        let first = RgbaImage::from_pixel(4, 2, Rgba([255, 0, 0, 255]));
        let second = RgbaImage::from_pixel(2, 3, Rgba([0, 0, 255, 255]));
        let first_path = dir.join("first.png");
        let second_path = dir.join("second.png");
        first.save(&first_path).unwrap();
        second.save(&second_path).unwrap();

        let merged = merge_images_vertical(&[first_path, second_path]).unwrap();
        let image = image::load_from_memory(&merged).unwrap();
        assert_eq!(image.width(), 4);
        assert_eq!(image.height(), 2 + 8 + 3);
        let _ = std::fs::remove_dir_all(dir);
    }
}
