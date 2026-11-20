use std::collections::HashSet;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::{GenericImageView, ImageFormat, Rgba, RgbaImage};

pub struct SavedImage {
    pub rel_path: String,
    pub width: u32,
    pub height: u32,
}

pub fn save_png(media_dir: &Path, hash: &str, png: &[u8]) -> Result<SavedImage, String> {
    let images = media_dir.join("images");
    std::fs::create_dir_all(&images).map_err(|e| e.to_string())?;
    let path = images.join(format!("{hash}.png"));
    std::fs::write(&path, png).map_err(|e| e.to_string())?;

    let img =
        image::load_from_memory_with_format(png, ImageFormat::Png).map_err(|e| e.to_string())?;
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

pub fn thumb_data_url(media_dir: &Path, rel_path: &str) -> Option<String> {
    let stem = Path::new(rel_path).file_stem()?.to_string_lossy();
    let path = media_dir.join("images").join(format!("{stem}_thumb.jpg"));
    let bytes = std::fs::read(path).ok()?;
    Some(format!("data:image/jpeg;base64,{}", STANDARD.encode(bytes)))
}

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
    let mut images = Vec::new();
    for path in paths {
        let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
        let image = image::load_from_memory(&bytes).map_err(|error| error.to_string())?;
        images.push(image.to_rgba8());
    }
    if images.is_empty() {
        return Err("没有可合并的图片".to_string());
    }

    const GAP: u32 = 8;
    let width = images.iter().map(|image| image.width()).max().unwrap_or(1);
    let height = images
        .iter()
        .map(|image| image.height())
        .sum::<u32>()
        .saturating_add(GAP.saturating_mul(images.len().saturating_sub(1) as u32));
    if width == 0 || height == 0 {
        return Err("图片尺寸无效".to_string());
    }

    let mut canvas = RgbaImage::from_pixel(width, height, Rgba([255, 255, 255, 255]));
    let mut y = 0i64;
    for image in images {
        let x = ((width - image.width()) / 2) as i64;
        image::imageops::overlay(&mut canvas, &image, x, y);
        y += image.height() as i64 + GAP as i64;
    }

    let mut buffer = Cursor::new(Vec::new());
    canvas
        .write_to(&mut buffer, ImageFormat::Png)
        .map_err(|error| error.to_string())?;
    Ok(buffer.into_inner())
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
