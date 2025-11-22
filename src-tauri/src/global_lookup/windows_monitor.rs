//! Windows-specific environment logging for global lookup debugging.
//!
//! This module provides diagnostic information about the window context
//! when a global lookup is triggered, helping debug capture issues.

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

/// Returns true if the window under the cursor appears to be Kindle.
pub fn is_kindle_under_cursor() -> bool {
    match target_window_process_path() {
        Some(path) => is_kindle_process(&path),
        None => false,
    }
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

fn is_kindle_process(process_path: &str) -> bool {
    let lower = process_path.to_ascii_lowercase();
    lower.ends_with("kindle.exe") || lower.contains("\\kindle\\") || lower.contains("amazon")
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
