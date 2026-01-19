use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn generate_minimal_ico_16x16_rgba() -> Vec<u8> {
    // Minimal ICO containing a single 16x16 32-bit BGRA DIB + empty AND mask.
    // This is only to satisfy `tauri-build`/`tauri-winres` during local builds.
    let pixels_len: usize = 16 * 16 * 4;
    let and_mask_row_stride: usize = 4; // (16/8)=2 bytes, padded to 4
    let and_mask_len: usize = and_mask_row_stride * 16;
    let dib_len: u32 = 40 + (pixels_len as u32) + (and_mask_len as u32);
    let image_offset: u32 = 6 + 16;

    let mut bytes = Vec::with_capacity((image_offset + dib_len) as usize);

    // ICONDIR
    bytes.extend_from_slice(&0u16.to_le_bytes()); // reserved
    bytes.extend_from_slice(&1u16.to_le_bytes()); // type (icon)
    bytes.extend_from_slice(&1u16.to_le_bytes()); // count

    // ICONDIRENTRY
    bytes.push(16); // width
    bytes.push(16); // height
    bytes.push(0); // color count
    bytes.push(0); // reserved
    bytes.extend_from_slice(&1u16.to_le_bytes()); // planes
    bytes.extend_from_slice(&32u16.to_le_bytes()); // bitcount
    bytes.extend_from_slice(&dib_len.to_le_bytes()); // bytes in res
    bytes.extend_from_slice(&image_offset.to_le_bytes()); // image offset

    // BITMAPINFOHEADER (40 bytes)
    bytes.extend_from_slice(&40u32.to_le_bytes()); // biSize
    bytes.extend_from_slice(&(16i32).to_le_bytes()); // biWidth
    bytes.extend_from_slice(&(32i32).to_le_bytes()); // biHeight (includes AND mask)
    bytes.extend_from_slice(&1u16.to_le_bytes()); // biPlanes
    bytes.extend_from_slice(&32u16.to_le_bytes()); // biBitCount
    bytes.extend_from_slice(&0u32.to_le_bytes()); // biCompression (BI_RGB)
    bytes.extend_from_slice(&(pixels_len as u32).to_le_bytes()); // biSizeImage
    bytes.extend_from_slice(&0i32.to_le_bytes()); // biXPelsPerMeter
    bytes.extend_from_slice(&0i32.to_le_bytes()); // biYPelsPerMeter
    bytes.extend_from_slice(&0u32.to_le_bytes()); // biClrUsed
    bytes.extend_from_slice(&0u32.to_le_bytes()); // biClrImportant

    // BGRA pixels (bottom-up). Use a solid orange-ish color.
    let (b, g, r, a) = (0x2Du8, 0x6Fu8, 0xFFu8, 0xFFu8);
    for _ in 0..(16 * 16) {
        bytes.push(b);
        bytes.push(g);
        bytes.push(r);
        bytes.push(a);
    }

    // AND mask (all 0 = fully opaque)
    bytes.resize(bytes.len() + and_mask_len, 0);
    bytes
}

fn ensure_windows_icon() -> io::Result<Option<PathBuf>> {
    if !cfg!(windows) {
        return Ok(None);
    }

    let icons_dir = Path::new("icons");
    let icon_path = icons_dir.join("icon.ico");

    if icon_path.exists() {
        return Ok(Some(icon_path));
    }

    fs::create_dir_all(icons_dir)?;
    fs::write(&icon_path, generate_minimal_ico_16x16_rgba())?;
    Ok(Some(icon_path))
}

fn main() {
    // Make local builds work out of the box on Windows.
    let _ = ensure_windows_icon();
    tauri_build::build()
}
