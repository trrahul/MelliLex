use async_trait::async_trait;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use log::warn;

use crate::{CaptureAttempt, CaptureError, CaptureRequest, CaptureResult, CaptureSource, CaptureStrategy};

/// Default capture region width in pixels (centered on cursor)
const CAPTURE_WIDTH: i32 = 400;
/// Default capture region height in pixels (centered on cursor)
const CAPTURE_HEIGHT: i32 = 200;

#[cfg(windows)]
use windows::Win32::Foundation::{HWND, POINT};
#[cfg(windows)]
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, RGBQUAD,
    SRCCOPY,
};
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

/// OCR-based capture strategy (Windows only).
///
/// Uses a small screen capture around the cursor and runs Tesseract via CLI.
#[derive(Clone, Copy, Default)]
pub struct OcrCaptureStrategy;

impl OcrCaptureStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CaptureStrategy for OcrCaptureStrategy {
    fn name(&self) -> &'static str {
        "ocr"
    }

    fn is_supported(&self) -> bool {
        cfg!(windows)
    }

    fn provides_ocr(&self) -> bool {
        true
    }

    async fn capture(&self, request: &CaptureRequest) -> Result<CaptureAttempt, CaptureError> {
        #[cfg(windows)]
        {
            let capture = capture_cursor_region(request, CAPTURE_WIDTH, CAPTURE_HEIGHT)
                .map_err(|err| CaptureError::internal(format!("ocr capture failed: {}", err)))?;

            let image_path = write_temp_bmp(&capture.bytes, capture.width, capture.height)
                .map_err(|err| CaptureError::internal(format!("ocr image write failed: {}", err)))?;

            let center_x = ((capture.cursor_x - capture.left) as f32)
                .clamp(0.0, capture.width as f32);
            let center_y = ((capture.cursor_y - capture.top) as f32)
                .clamp(0.0, capture.height as f32);

            let output = run_tesseract(&image_path, center_x, center_y)
                .map_err(|err| CaptureError::internal(format!("tesseract failed: {}", err)))?;

            let _ = std::fs::remove_file(&image_path);

            let text = normalize_ocr_text(&output);
            if text.is_empty() {
                return Ok(CaptureAttempt::NoData);
            }

            return Ok(CaptureAttempt::Success(CaptureResult::new(
                text,
                CaptureSource::Ocr,
            )));
        }

        #[cfg(not(windows))]
        {
            let _ = request;
            Ok(CaptureAttempt::NoData)
        }
    }
}

#[cfg(windows)]
struct CaptureBuffer {
    bytes: Vec<u8>,
    width: i32,
    height: i32,
    left: i32,
    top: i32,
    cursor_x: i32,
    cursor_y: i32,
}

#[cfg(windows)]
fn capture_cursor_region(
    request: &CaptureRequest,
    width: i32,
    height: i32,
) -> Result<CaptureBuffer, String> {
    unsafe {
        let mut point = POINT::default();
        if let Some(cursor) = request.cursor {
            point.x = cursor.x;
            point.y = cursor.y;
        } else if GetCursorPos(&mut point).is_err() {
            return Err("failed to read cursor position".to_string());
        }

        let left = point.x - width / 2;
        let top = point.y - height / 2;

        let screen_dc = GetDC(HWND(std::ptr::null_mut()));
        if screen_dc.0 == std::ptr::null_mut() {
            return Err("failed to get screen DC".to_string());
        }

        let mem_dc = CreateCompatibleDC(screen_dc);
        if mem_dc.0 == std::ptr::null_mut() {
            ReleaseDC(HWND(std::ptr::null_mut()), screen_dc);
            return Err("failed to create compatible DC".to_string());
        }

        let bitmap = CreateCompatibleBitmap(screen_dc, width, height);
        if bitmap.0 == std::ptr::null_mut() {
            let _ = DeleteDC(mem_dc);
            ReleaseDC(HWND(std::ptr::null_mut()), screen_dc);
            return Err("failed to create compatible bitmap".to_string());
        }

        let old = SelectObject(mem_dc, bitmap);

        if BitBlt(mem_dc, 0, 0, width, height, screen_dc, left, top, SRCCOPY).is_err() {
            SelectObject(mem_dc, old);
            let _ = DeleteObject(bitmap);
            let _ = DeleteDC(mem_dc);
            ReleaseDC(HWND(std::ptr::null_mut()), screen_dc);
            return Err("BitBlt failed".to_string());
        }

        let info_header = BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height, // top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0 as u32,
            biSizeImage: (width * height * 4) as u32,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        };

        let mut info = BITMAPINFO {
            bmiHeader: info_header,
            bmiColors: [RGBQUAD::default(); 1],
        };

        let mut buffer = vec![0u8; (width * height * 4) as usize];
        let lines = GetDIBits(
            mem_dc,
            bitmap,
            0,
            height as u32,
            Some(buffer.as_mut_ptr() as _),
            &mut info,
            DIB_RGB_COLORS,
        );
        SelectObject(mem_dc, old);
        let _ = DeleteObject(bitmap);
        let _ = DeleteDC(mem_dc);
        ReleaseDC(HWND(std::ptr::null_mut()), screen_dc);

        if lines == 0 {
            return Err("GetDIBits failed".to_string());
        }

        Ok(CaptureBuffer {
            bytes: buffer,
            width,
            height,
            left,
            top,
            cursor_x: point.x,
            cursor_y: point.y,
        })
    }
}

#[cfg(windows)]
fn write_temp_bmp(bytes: &[u8], width: i32, height: i32) -> Result<PathBuf, String> {
    let mut path = std::env::temp_dir();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_millis();
    path.push(format!("mellilex-ocr-{}.bmp", ts));

    let mut file = File::create(&path).map_err(|err| err.to_string())?;

    let header_size = 14u32;
    let dib_size = 40u32;
    let image_size = (width * height * 4) as u32;
    let file_size = header_size + dib_size + image_size;

    let mut header = Vec::with_capacity(file_size as usize);
    header.extend_from_slice(b"BM");
    header.extend_from_slice(&file_size.to_le_bytes());
    header.extend_from_slice(&[0u8; 4]);
    header.extend_from_slice(&(header_size + dib_size).to_le_bytes());

    header.extend_from_slice(&dib_size.to_le_bytes());
    header.extend_from_slice(&width.to_le_bytes());
    header.extend_from_slice(&(-height).to_le_bytes());
    header.extend_from_slice(&1u16.to_le_bytes());
    header.extend_from_slice(&32u16.to_le_bytes());
    header.extend_from_slice(&0u32.to_le_bytes());
    header.extend_from_slice(&image_size.to_le_bytes());
    header.extend_from_slice(&0u32.to_le_bytes());
    header.extend_from_slice(&0u32.to_le_bytes());
    header.extend_from_slice(&0u32.to_le_bytes());
    header.extend_from_slice(&0u32.to_le_bytes());

    file.write_all(&header).map_err(|err| err.to_string())?;
    file.write_all(bytes).map_err(|err| err.to_string())?;

    Ok(path)
}

#[cfg(windows)]
fn run_tesseract(path: &PathBuf, center_x: f32, center_y: f32) -> Result<String, String> {
    let (exe, tessdata_dir) = resolve_tesseract_paths();
    if let Some(ref exe_path) = exe {
        if !exe_path.exists() {
            warn!("[OCR] Tesseract binary does not exist at {:?}", exe_path);
        }
    }
    if let Some(ref dir) = tessdata_dir {
        if !dir.exists() {
            warn!("[OCR] Tessdata dir does not exist at {:?}", dir);
        }
    }
    let mut command = if let Some(ref exe_path) = exe {
        Command::new(exe_path)
    } else {
        Command::new("tesseract")
    };

    let output_base = build_output_base()?;
    
    // Build args: tesseract input output -l eng --oem 1 --dpi 300 --psm 6 -c tessedit_create_tsv=1
    command.arg(path).arg(&output_base);

    if let Some(dir) = tessdata_dir.as_ref() {
        command.env("TESSDATA_PREFIX", dir);
        command.arg("--tessdata-dir").arg(dir);
    }

    let output = command
        .arg("-l")
        .arg("eng")
        .arg("--oem")
        .arg("1")
        .arg("--dpi")
        .arg("300")
        .arg("--psm")
        .arg("6")
        .arg("-c")
        .arg("tessedit_create_tsv=1")
        .output()
        .map_err(|err| err.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("[OCR] Tesseract failed: {}", stderr.trim());
        return Err(stderr.trim().to_string());
    }

    let tsv_path = output_base.with_extension("tsv");
    
    let tsv = std::fs::read_to_string(&tsv_path).map_err(|err| {
        warn!("[OCR] Failed to read TSV file: {}", err);
        err.to_string()
    })?;
    let _ = std::fs::remove_file(&tsv_path);

    if let Some(word) = extract_best_word_from_tsv(&tsv, center_x, center_y) {
        return Ok(word);
    }

    run_tesseract_stdout(path, tessdata_dir, exe)
}

#[cfg(windows)]
fn run_tesseract_stdout(
    path: &PathBuf,
    tessdata_dir: Option<PathBuf>,
    exe: Option<PathBuf>,
) -> Result<String, String> {
    let mut command = if let Some(exe_path) = exe {
        Command::new(exe_path)
    } else {
        Command::new("tesseract")
    };

    command.arg(path).arg("stdout");

    if let Some(dir) = tessdata_dir.as_ref() {
        command.env("TESSDATA_PREFIX", dir);
        command.arg("--tessdata-dir").arg(dir);
    }

    let output = command
        .arg("-l")
        .arg("eng")
        .arg("--oem")
        .arg("1")
        .arg("--dpi")
        .arg("300")
        .arg("--psm")
        .arg("8")
        .output()
        .map_err(|err| err.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().to_string());
    }

    let text = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(extract_first_token(&text))
}

#[cfg(windows)]
fn extract_first_token(raw: &str) -> String {
    let normalized = normalize_ocr_text(raw);
    if normalized.is_empty() {
        return String::new();
    }

    let mut token = String::new();
    for ch in normalized.chars() {
        if ch.is_alphanumeric() || ch == '-' || ch == '\'' {
            token.push(ch);
        } else if !token.is_empty() {
            break;
        }
    }

    token
}

#[cfg(windows)]
fn build_output_base() -> Result<PathBuf, String> {
    let mut path = std::env::temp_dir();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_millis();
    path.push(format!("mellilex-ocr-{}", ts));
    Ok(path)
}

#[cfg(windows)]
fn extract_best_word_from_tsv(tsv: &str, center_x: f32, center_y: f32) -> Option<String> {
    let mut best: Option<(f32, String)> = None;

    for (index, line) in tsv.lines().enumerate() {
        if index == 0 {
            continue;
        }
        let columns: Vec<&str> = line.split('\t').collect();
        if columns.len() < 12 {
            continue;
        }

        let text = columns[11].trim();
        if text.is_empty() {
            continue;
        }

        let conf: f32 = columns[10].parse().unwrap_or(-1.0);
        // Require minimum confidence of 50% to filter garbage
        if conf < 50.0 {
            continue;
        }

        let left: f32 = columns[6].parse().unwrap_or(0.0);
        let top: f32 = columns[7].parse().unwrap_or(0.0);
        let width: f32 = columns[8].parse().unwrap_or(0.0);
        let height: f32 = columns[9].parse().unwrap_or(0.0);

        let word = normalize_ocr_text(text);
        if word.is_empty() {
            continue;
        }
        
        // Skip words that are mostly non-alphabetic (likely OCR garbage)
        let alpha_count = word.chars().filter(|c| c.is_alphabetic()).count();
        if alpha_count < word.len() / 2 {
            continue;
        }

        let word_center_x = left + width / 2.0;
        let word_center_y = top + height / 2.0;
        let distance = (word_center_x - center_x).powi(2) + (word_center_y - center_y).powi(2);

        match &best {
            Some((best_distance, _)) if distance >= *best_distance => {}
            _ => best = Some((distance, word)),
        }
    }

    best.map(|(_, word)| word)
}

#[cfg(windows)]
fn resolve_tesseract_paths() -> (Option<PathBuf>, Option<PathBuf>) {
    if let Ok(path) = std::env::var("MELLILEX_TESSERACT_PATH") {
        let exe = PathBuf::from(path);
        let tessdata = exe.parent().map(|dir| dir.join("tessdata"));
        return (Some(exe), tessdata);
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let bundled = exe_dir.join("resources").join("tesseract").join("tesseract.exe");
            if bundled.exists() {
                let tessdata = bundled.parent().map(|dir| dir.join("tessdata"));
                return (Some(bundled), tessdata);
            }
        }
    }

    let dev_paths = [
        PathBuf::from("resources").join("tesseract").join("tesseract.exe"),
        PathBuf::from("src-tauri")
            .join("resources")
            .join("tesseract")
            .join("tesseract.exe"),
    ];

    for exe in dev_paths {
        if exe.exists() {
            let tessdata = exe.parent().map(|dir| dir.join("tessdata"));
            return (Some(exe), tessdata);
        }
    }

    (None, None)
}

#[cfg(windows)]
fn normalize_ocr_text(raw: &str) -> String {
    let cleaned = raw
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    let joined = cleaned.join(" ");
    let trimmed = joined.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let stripped = trimmed
        .trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '\'')
        .to_string();

    if stripped.len() > 200 {
        return stripped.chars().take(200).collect();
    }
    stripped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_ocr_text_trims_and_limits() {
        let raw = "\n  Hello world  \n\n";
        assert_eq!(normalize_ocr_text(raw), "Hello world");

        let long = "a".repeat(300);
        assert_eq!(normalize_ocr_text(&long).len(), 200);
    }

    #[cfg(windows)]
    #[test]
    fn resolve_tesseract_paths_prefers_env() {
        let temp = std::env::temp_dir().join("mellilex-ocr-test");
        let _ = std::fs::create_dir_all(&temp);
        let exe = temp.join("tesseract.exe");
        let _ = std::fs::write(&exe, b"");

        std::env::set_var("MELLILEX_TESSERACT_PATH", exe.to_string_lossy().to_string());
        let (resolved, tessdata) = resolve_tesseract_paths();
        std::env::remove_var("MELLILEX_TESSERACT_PATH");

        assert_eq!(resolved, Some(exe));
        assert_eq!(tessdata, Some(temp.join("tessdata")));
    }

    #[cfg(windows)]
    #[test]
    fn tsv_pick_best_word_near_center() {
        let tsv = "level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext\n\
5\t1\t1\t1\t1\t1\t10\t10\t40\t20\t90\thello\n\
5\t1\t1\t1\t1\t2\t150\t60\t40\t20\t88\tworld";
        let word = extract_best_word_from_tsv(tsv, 160.0, 70.0).unwrap();
        assert_eq!(word, "world");
    }

    #[cfg(windows)]
    #[test]
    fn extract_first_token_returns_word() {
        let raw = "Hello world and more";
        assert_eq!(extract_first_token(raw), "Hello");

        let raw = "***quoted-word***";
        assert_eq!(extract_first_token(raw), "quoted-word");
    }
}
