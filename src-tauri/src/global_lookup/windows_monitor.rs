//! Windows-specific environment logging for global lookup debugging.
//!
//! This module provides diagnostic information about the window context
//! when a global lookup is triggered, helping debug capture issues.

use uiautomation::UIAutomation;
use uiautomation::types::TreeScope;
use uiautomation::patterns::UIValuePattern;
use windows::{
    core::PWSTR,
    Win32::{
        Foundation::{CloseHandle, HWND, POINT},
        System::Threading::{
            OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
            PROCESS_QUERY_LIMITED_INFORMATION,
        },
        UI::WindowsAndMessaging::{
            GetClassNameW, GetCursorPos, GetForegroundWindow, GetParent, GetWindowTextW,
            GetWindowThreadProcessId, WindowFromPoint,
        },
    },
};

/// Logs a snapshot of the Windows environment at capture time.
///
/// Captures cursor position, target window under cursor, and foreground window details.
/// This information is invaluable for debugging why certain applications fail to capture.
pub fn log_environment_snapshot() {
    unsafe {
        let mut point = POINT::default();
        if GetCursorPos(&mut point).is_ok() {
            log::debug!(
                "[GlobalLookup][Env] Cursor at screen coordinates x={}, y={}",
                point.x,
                point.y
            );
        }

        let hwnd = WindowFromPoint(point);
        if !hwnd.0.is_null() {
            log_window_details("[GlobalLookup][Env] Target window", hwnd);
        }

        let fg = GetForegroundWindow();
        if !fg.0.is_null() {
            log_window_details("[GlobalLookup][Env] Foreground window", fg);
        }
    }
}

/// Returns true if the window under the cursor belongs to an application
/// that requires OCR-based capture (no UIA TextPattern support).
pub fn should_prefer_ocr() -> bool {
    // 1. Check the Win32 window process (works when the target app is topmost).
    if let Some(path) = target_window_process_path() {
        if needs_ocr_capture(&path) {
            return true;
        }
        if is_browser_process(&path) {
            if check_browser_url_for_ocr() {
                return true;
            }
        }
    }

    // 2. Use UIA element_from_point which sees through overlapping windows
    //    (e.g. when MelliLex is on top of the browser).
    if let Some(result) = uia_probe_at_cursor() {
        if needs_ocr_by_element_class(&result.element_class) {
            log::info!(
                "[GlobalLookup][Env] UIA element class '{}' matches OCR-preferred pattern",
                result.element_class
            );
            return true;
        }
        if let Some(ref process) = result.process_path {
            if needs_ocr_capture(process) {
                return true;
            }
            if is_browser_process(process) {
                if check_browser_url_for_ocr_in_process(process) {
                    return true;
                }
            }
        }
    }

    false
}

struct UiaProbeResult {
    element_class: String,
    process_path: Option<String>,
}

/// Uses UIA element_from_point to get the element under the cursor.
/// Unlike Win32 WindowFromPoint, UIA sees through overlapping windows.
fn uia_probe_at_cursor() -> Option<UiaProbeResult> {
    let automation = UIAutomation::new().ok()?;

    let point = unsafe {
        let mut p = POINT::default();
        GetCursorPos(&mut p).ok()?;
        p
    };

    let uia_point = uiautomation::types::Point::new(point.x, point.y);
    let element = automation.element_from_point(uia_point).ok()?;
    let element_class = element.get_classname().unwrap_or_default();

    // Walk up to find the top-level window's process
    let process_path = get_element_process_path(&element);

    Some(UiaProbeResult {
        element_class,
        process_path,
    })
}

/// Gets the process path of the top-level window owning a UIA element.
fn get_element_process_path(element: &uiautomation::UIElement) -> Option<String> {
    let pid = element.get_process_id().ok()?;
    let path = get_process_path(pid as u32);
    if path == "<unknown>" || path == "<unopened>" {
        None
    } else {
        Some(path)
    }
}

/// UIA element class names that indicate OCR is needed.
fn needs_ocr_by_element_class(class: &str) -> bool {
    let lower = class.to_ascii_lowercase();
    // Kindle Cloud Reader (read.amazon.*) uses "kr-" prefixed CSS class names
    lower.starts_with("kr-")
}

/// Known browser executables.
fn is_browser_process(process_path: &str) -> bool {
    let lower = process_path.to_ascii_lowercase();
    lower.ends_with("chrome.exe")
        || lower.ends_with("brave.exe")
        || lower.ends_with("msedge.exe")
        || lower.ends_with("firefox.exe")
        || lower.ends_with("opera.exe")
        || lower.ends_with("vivaldi.exe")
}

/// URLs that need OCR because the site renders text as images/canvas.
fn needs_ocr_by_url(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.contains("read.amazon")
}

/// Checks if the foreground browser window's URL needs OCR.
fn check_browser_url_for_ocr() -> bool {
    if let Some(url) = browser_address_bar_url_from_foreground() {
        if needs_ocr_by_url(&url) {
            log::info!(
                "[GlobalLookup][Env] Browser URL matches OCR-preferred site: {}",
                url
            );
            return true;
        }
    }
    false
}

/// Checks if any visible window of the given browser process has an OCR-needing URL.
fn check_browser_url_for_ocr_in_process(process_path: &str) -> bool {
    // Try foreground first (cheap)
    if check_browser_url_for_ocr() {
        return true;
    }
    // If foreground isn't the browser (MelliLex is on top), find a browser window by process name.
    if let Some(url) = browser_address_bar_url_by_process(process_path) {
        if needs_ocr_by_url(&url) {
            log::info!(
                "[GlobalLookup][Env] Browser URL (behind overlay) matches OCR-preferred site: {}",
                url
            );
            return true;
        }
    }
    false
}

/// Reads the URL from the foreground window's address bar via UIA ValuePattern.
fn browser_address_bar_url_from_foreground() -> Option<String> {
    let fg = unsafe { GetForegroundWindow() };
    if fg.0.is_null() {
        return None;
    }
    browser_address_bar_url_from_hwnd(fg)
}

/// Finds a browser window by process name and reads its address bar URL.
fn browser_address_bar_url_by_process(_process_path: &str) -> Option<String> {
    // Get the PID of the browser from the UIA element's process
    let automation = UIAutomation::new().ok()?;

    let point = unsafe {
        let mut p = POINT::default();
        GetCursorPos(&mut p).ok()?;
        p
    };

    let uia_point = uiautomation::types::Point::new(point.x, point.y);
    let element = automation.element_from_point(uia_point).ok()?;
    let target_pid = element.get_process_id().ok()? as u32;

    // Find the top-level window for this PID
    find_top_level_window_for_pid(target_pid)
        .and_then(|hwnd| browser_address_bar_url_from_hwnd(hwnd))
}

/// Finds the first top-level visible window owned by the given PID.
fn find_top_level_window_for_pid(target_pid: u32) -> Option<HWND> {
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, IsWindowVisible, GetWindow, GW_OWNER,
    };

    // Use a raw isize to pass the result back via thread-local (EnumWindows is synchronous).
    std::thread_local! {
        static FOUND_HWND: std::cell::Cell<isize> = const { std::cell::Cell::new(0) };
    }

    FOUND_HWND.with(|c| c.set(0));

    unsafe extern "system" fn callback(hwnd: HWND, lparam: windows::Win32::Foundation::LPARAM) -> windows::Win32::Foundation::BOOL {
        let target_pid = lparam.0 as u32;
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == target_pid && IsWindowVisible(hwnd).as_bool() {
            // Skip owned windows (popups, tooltips)
            let owner = GetWindow(hwnd, GW_OWNER);
            if matches!(owner, Ok(h) if h.0.is_null()) || owner.is_err() {
                FOUND_HWND.with(|c| c.set(hwnd.0 as isize));
                return windows::Win32::Foundation::FALSE; // stop enumeration
            }
        }
        windows::Win32::Foundation::TRUE
    }

    unsafe {
        let _ = EnumWindows(
            Some(callback),
            windows::Win32::Foundation::LPARAM(target_pid as isize),
        );
    }

    let raw = FOUND_HWND.with(|c| c.get());
    if raw == 0 {
        None
    } else {
        Some(HWND(raw as *mut _))
    }
}

/// Reads the URL from a specific browser window's address bar via UIA.
fn browser_address_bar_url_from_hwnd(hwnd: HWND) -> Option<String> {
    let automation = UIAutomation::new().ok()?;

    // Bridge HWND across different `windows` crate versions by transmuting the raw pointer.
    // Both are repr(transparent) wrappers around isize.
    let handle: uiautomation::types::Handle = unsafe { std::mem::transmute(hwnd) };
    let window = automation.element_from_handle(handle).ok()?;

    // ControlType.Edit = 50004
    let edit_type = automation
        .create_property_condition(
            uiautomation::types::UIProperty::ControlType,
            uiautomation::variants::Variant::from(50004i32),
            None,
        )
        .ok()?;

    let edits = window
        .find_all(TreeScope::Descendants, &edit_type)
        .ok()?;

    for edit in edits {
        if let Ok(value_pattern) = edit.get_pattern::<UIValuePattern>() {
            if let Ok(value) = value_pattern.get_value() {
                let trimmed = value.trim();
                // Looks like a URL or domain (contains a dot and no spaces)
                if !trimmed.is_empty()
                    && trimmed.contains('.')
                    && !trimmed.contains(' ')
                {
                    return Some(trimmed.to_string());
                }
            }
        }
    }
    None
}

/// Logs detailed information about a specific window.
fn log_window_details(label: &str, hwnd: HWND) {
    unsafe {
        let mut class_buf = [0u16; 256];
        let class_len = GetClassNameW(hwnd, &mut class_buf);
        let class_name = String::from_utf16_lossy(&class_buf[..class_len as usize]);

        let mut title_buf = [0u16; 512];
        let title_len = GetWindowTextW(hwnd, &mut title_buf);
        let title = String::from_utf16_lossy(&title_buf[..title_len as usize]);

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        let process_path = get_process_path(pid);

        let hierarchy = collect_window_hierarchy(hwnd);

        log::debug!(
            "{} HWND={:?}, class='{}', title='{}', pid={}, process='{}', hierarchy={:?}",
            label,
            hwnd,
            class_name,
            title,
            pid,
            process_path,
            hierarchy
        );
    }
}

fn target_window_process_path() -> Option<String> {
    unsafe {
        let mut point = POINT::default();
        if GetCursorPos(&mut point).is_err() {
            return None;
        }

        let hwnd = WindowFromPoint(point);
        if hwnd.0.is_null() {
            return None;
        }

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        let process_path = get_process_path(pid);
        if process_path == "<unknown>" || process_path == "<unopened>" {
            None
        } else {
            Some(process_path)
        }
    }
}

/// Applications whose text rendering doesn't expose UIA TextPattern,
/// requiring OCR-based capture instead.
fn needs_ocr_capture(process_path: &str) -> bool {
    let lower = process_path.to_ascii_lowercase();
    // Kindle
    lower.ends_with("kindle.exe")
        || lower.contains("\\kindle\\")
    // Adobe Acrobat / Reader
        || lower.ends_with("acrobat.exe")
        || lower.ends_with("acrord32.exe")
        || lower.ends_with("acrord64.exe")
    // Foxit PDF
        || lower.ends_with("foxitpdfreader.exe")
        || lower.ends_with("foxitreader.exe")
        || lower.ends_with("foxitpdfeditor.exe")
    // SumatraPDF
        || lower.ends_with("sumatrapdf.exe")
    // Calibre e-book viewer (Qt / QtWebEngine — no UIA TextPattern)
        || lower.ends_with("ebook-viewer.exe")
}

/// Gets the full path of a process by PID.
fn get_process_path(pid: u32) -> String {
    if pid == 0 {
        return "<unknown>".to_string();
    }

    unsafe {
        let process_handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid);
        if let Ok(handle) = process_handle {
            let mut buffer = [0u16; 260];
            let mut size = buffer.len() as u32;
            let result = QueryFullProcessImageNameW(
                handle,
                PROCESS_NAME_WIN32,
                PWSTR(buffer.as_mut_ptr()),
                &mut size,
            );
            let _ = CloseHandle(handle);
            if result.is_ok() {
                String::from_utf16_lossy(&buffer[..size as usize])
            } else {
                "<unknown>".to_string()
            }
        } else {
            "<unopened>".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::needs_ocr_capture;

    #[test]
    fn detects_calibre_viewer() {
        assert!(needs_ocr_capture(r"C:\Program Files\Calibre2\ebook-viewer.exe"));
        // Case-insensitive
        assert!(needs_ocr_capture(r"C:\PROGRAM FILES\CALIBRE2\EBOOK-VIEWER.EXE"));
        // Main library GUI is intentionally not OCR-preferred
        assert!(!needs_ocr_capture(r"C:\Program Files\Calibre2\calibre.exe"));
    }

    #[test]
    fn still_detects_existing_ocr_apps() {
        assert!(needs_ocr_capture(r"C:\Program Files\Amazon\Kindle\Kindle.exe"));
        assert!(needs_ocr_capture(r"C:\Program Files\Adobe\Acrobat\Acrobat.exe"));
        assert!(needs_ocr_capture(r"C:\Foxit\FoxitPDFReader.exe"));
        assert!(needs_ocr_capture(r"C:\tools\SumatraPDF.exe"));
    }

    #[test]
    fn ignores_unrelated_processes() {
        assert!(!needs_ocr_capture(r"C:\Windows\notepad.exe"));
        assert!(!needs_ocr_capture(r"C:\Program Files\Microsoft VS Code\Code.exe"));
    }
}

/// Collects the window class hierarchy from a window up to its root.
fn collect_window_hierarchy(mut hwnd: HWND) -> Vec<String> {
    unsafe {
        let mut nodes = Vec::new();
        loop {
            let mut class_buf = [0u16; 256];
            let len = GetClassNameW(hwnd, &mut class_buf);
            if len == 0 {
                break;
            }
            nodes.push(String::from_utf16_lossy(&class_buf[..len as usize]));
            let parent = match GetParent(hwnd) {
                Ok(parent) => parent,
                Err(_) => break,
            };
            if parent.0.is_null() {
                break;
            }
            hwnd = parent;
        }
        nodes
    }
}
