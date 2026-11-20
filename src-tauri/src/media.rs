use std::io::Cursor;
use std::path::Path;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::{GenericImageView, ImageFormat};

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

    let img = image::load_from_memory_with_format(png, ImageFormat::Png).map_err(|e| e.to_string())?;
    let (width, height) = img.dimensions();
    let thumb = img.thumbnail(300, 300).to_rgb8();
    let mut buffer = Cursor::new(Vec::new());
    if thumb.write_to(&mut buffer, ImageFormat::Jpeg).is_ok() {
        let _ = std::fs::write(images.join(format!("{hash}_thumb.jpg")), buffer.into_inner());
    }

    Ok(SavedImage {
        rel_path: format!("images/{hash}.png"),
        width,
        height,
    })
}

pub fn thumb_data_url(media_dir: &Path, rel_path: &str) -> Option<String> {
    let stem = Path::new(rel_path).file_stem()?.to_string_lossy();
    let path = media_dir
        .join("images")
        .join(format!("{stem}_thumb.jpg"));
    let bytes = std::fs::read(path).ok()?;
    Some(format!("data:image/jpeg;base64,{}", STANDARD.encode(bytes)))
}
