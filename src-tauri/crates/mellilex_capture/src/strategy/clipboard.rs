use std::ptr;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use std::time::Duration;
use tokio::task::spawn_blocking;
use tracing::{debug, warn};
use widestring::U16CStr;
use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL, HWND};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, EnumClipboardFormats, GetClipboardData, OpenClipboard,
    SetClipboardData,
};
use windows::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE,
};
use windows::Win32::System::Ole::CF_UNICODETEXT;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_C, VK_CONTROL,
};

use crate::{
    CaptureAttempt, CaptureError, CaptureRequest, CaptureResult, CaptureSource, CaptureStrategy,
};

/// Clipboard-based capture strategy (simulate Ctrl+C and read clipboard text).
#[derive(Clone, Copy)]
pub struct ClipboardCaptureStrategy;

impl ClipboardCaptureStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ClipboardCaptureStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CaptureStrategy for ClipboardCaptureStrategy {
    fn name(&self) -> &'static str {
        "clipboard"
    }

    fn is_supported(&self) -> bool {
        cfg!(windows)
    }

    async fn capture(&self, request: &CaptureRequest) -> Result<CaptureAttempt, CaptureError> {
        #[cfg(windows)]
        {
            let strategy = *self;
            let request = request.clone();
            match spawn_blocking(move || strategy.capture_windows(&request))
                .await
                .map_err(|err| {
                    CaptureError::internal(format!("clipboard capture panicked: {}", err))
                })?
                .map_err(|e| CaptureError::internal(format!("clipboard capture failed: {}", e)))?
            {
                Some(text) => Ok(CaptureAttempt::Success(CaptureResult::new(
                    text,
                    CaptureSource::Clipboard,
                ))),
                None => Ok(CaptureAttempt::NoData),
            }
        }

        #[cfg(not(windows))]
        {
            Ok(CaptureAttempt::NoData)
        }
    }
}

#[cfg(windows)]
impl ClipboardCaptureStrategy {
    fn capture_windows(&self, _request: &CaptureRequest) -> Result<Option<String>> {
        debug!("[Clipboard] Attempting capture via Ctrl+C");

        let original_text = self.read_clipboard_text().unwrap_or(None);
        let snapshot = match ClipboardSnapshot::capture() {
            Ok(snapshot) => snapshot,
            Err(err) => {
                debug!("[Clipboard] Unable to snapshot clipboard: {err}");
                return Ok(None);
            }
        };
        let _restorer = ClipboardRestorer::new(snapshot);

        if let Err(err) = self.simulate_ctrl_c() {
            debug!("[Clipboard] Failed to send Ctrl+C: {err}");
            return Ok(None);
        }
        std::thread::sleep(Duration::from_millis(120));

        let captured = match self.read_clipboard_text() {
            Ok(value) => value,
            Err(err) => {
                debug!("[Clipboard] Failed to read clipboard after Ctrl+C: {err}");
                None
            }
        };

        Ok(self.sanitize_capture(original_text.as_ref(), captured.as_deref()))
    }

    fn read_clipboard_text(&self) -> Result<Option<String>> {
        let _guard = ClipboardGuard::new()?;

        unsafe {
            let handle = GetClipboardData(CF_UNICODETEXT.0 as u32)?;
            if handle.0.is_null() {
                return Ok(None);
            }
            let hglobal = HGLOBAL(handle.0);
            let locked = GlobalLock(hglobal);
            if locked.is_null() {
                return Ok(None);
            }

            let slice = U16CStr::from_ptr_str(locked as *const u16);
            let text = slice.to_string_lossy();
            let _ = GlobalUnlock(hglobal);
            Ok(Some(text))
        }
    }

    fn sanitize_capture(&self, original: Option<&String>, captured: Option<&str>) -> Option<String> {
        let text = captured?.trim();
        if text.is_empty() {
            return None;
        }

        if let Some(original_text) = original {
            if text == original_text.trim() {
                debug!("[Clipboard] Clipboard content unchanged; ignoring");
                return None;
            }
        }

        Some(text.to_string())
    }

    fn simulate_ctrl_c(&self) -> Result<()> {
        unsafe {
            let inputs: [INPUT; 4] = [
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VK_CONTROL,
                            ..Default::default()
                        },
                    },
                },
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VK_C,
                            ..Default::default()
                        },
                    },
                },
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VK_C,
                            dwFlags: KEYEVENTF_KEYUP,
                            ..Default::default()
                        },
                    },
                },
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VK_CONTROL,
                            dwFlags: KEYEVENTF_KEYUP,
                            ..Default::default()
                        },
                    },
                },
            ];

            let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            if sent != inputs.len() as u32 {
                return Err(anyhow!("SendInput failed to send all events"));
            }
        }
        Ok(())
    }

}

#[cfg(windows)]
struct ClipboardGuard;

#[cfg(windows)]
impl ClipboardGuard {
    fn new() -> Result<Self> {
        unsafe {
            OpenClipboard(HWND(ptr::null_mut()))?;
            Ok(Self)
        }
    }
}

#[cfg(windows)]
impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}

#[cfg(windows)]
struct ClipboardSnapshot {
    entries: Vec<ClipboardEntry>,
}

#[cfg(windows)]
impl ClipboardSnapshot {
    fn capture() -> Result<Self> {
        let _guard = ClipboardGuard::new()?;
        let mut entries = Vec::new();

        unsafe {
            let mut format = EnumClipboardFormats(0);
            if format == 0 {
                return Ok(Self { entries });
            }
            loop {
                entries.push(ClipboardEntry::from_format(format)?);
                format = EnumClipboardFormats(format);
                if format == 0 {
                    break;
                }
            }
        }

        Ok(Self { entries })
    }

    fn restore(&self) -> Result<()> {
        let _guard = ClipboardGuard::new()?;
        unsafe {
            EmptyClipboard()?;
            for entry in &self.entries {
                entry.restore()?;
            }
        }
        Ok(())
    }
}

#[cfg(windows)]
struct ClipboardEntry {
    format: u32,
    data: Vec<u8>,
}

#[cfg(windows)]
impl ClipboardEntry {
    fn from_format(format: u32) -> Result<Self> {
        unsafe {
            let handle = GetClipboardData(format)?;
            if handle.0.is_null() {
                return Err(anyhow!("clipboard handle for format {format} is null"));
            }

            let hglobal = HGLOBAL(handle.0);
            let size = GlobalSize(hglobal);
            if size == 0 {
                return Err(anyhow!(
                    "clipboard format {format} is not backed by movable global memory"
                ));
            }

            let locked = GlobalLock(hglobal);
            if locked.is_null() {
                return Err(anyhow!(
                    "clipboard format {format} could not be locked (unsupported)"
                ));
            }

            let slice = std::slice::from_raw_parts(locked as *const u8, size);
            let mut data = vec![0u8; size];
            data.copy_from_slice(slice);
            let _ = GlobalUnlock(hglobal);

            Ok(Self { format, data })
        }
    }

    unsafe fn restore(&self) -> Result<()> {
        if self.data.is_empty() {
            return Ok(());
        }

        let handle = GlobalAlloc(GMEM_MOVEABLE, self.data.len())
            .context("GlobalAlloc failed when restoring clipboard")?;
        let locked = GlobalLock(handle);
        if locked.is_null() {
            let _ = GlobalFree(handle);
            return Err(anyhow!("GlobalLock failed when restoring clipboard"));
        }

        ptr::copy_nonoverlapping(self.data.as_ptr(), locked as *mut u8, self.data.len());
        let _ = GlobalUnlock(handle);
        match SetClipboardData(self.format, HANDLE(handle.0)) {
            Ok(_) => Ok(()),
            Err(err) => {
                let _ = GlobalFree(handle);
                Err(anyhow!(
                    "SetClipboardData failed when restoring clipboard: {err}"
                ))
            }
        }
    }
}

#[cfg(windows)]
struct ClipboardRestorer {
    snapshot: ClipboardSnapshot,
}

#[cfg(windows)]
impl ClipboardRestorer {
    fn new(snapshot: ClipboardSnapshot) -> Self {
        Self { snapshot }
    }
}

#[cfg(windows)]
impl Drop for ClipboardRestorer {
    fn drop(&mut self) {
        if let Err(err) = self.snapshot.restore() {
            warn!("[Clipboard] Failed to restore clipboard contents: {err}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strategy_name() {
        assert_eq!(ClipboardCaptureStrategy::new().name(), "clipboard");
    }
}
