#![cfg(windows)]

use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use windows::Win32::{
    Foundation::{HANDLE, HINSTANCE, LPARAM, LRESULT, WPARAM},
    UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_CONTROL},
    UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, MsgWaitForMultipleObjects, PeekMessageW,
        SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, HHOOK, MSG, PM_REMOVE,
        QS_ALLINPUT, WH_MOUSE_LL, WM_QUIT, WM_RBUTTONDOWN, WM_RBUTTONUP,
    },
};

type HookCallback = Arc<dyn Fn() + Send + Sync + 'static>;

static CALLBACK: Lazy<Mutex<Option<HookCallback>>> = Lazy::new(|| Mutex::new(None));

pub struct CtrlRightClickHook {
    stop_tx: mpsc::Sender<()>,
    thread: Option<thread::JoinHandle<()>>,
}

impl Drop for CtrlRightClickHook {
    fn drop(&mut self) {
        let _ = self.stop_tx.send(());
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

pub fn start_ctrl_right_click_hook<F>(callback: F) -> Result<CtrlRightClickHook>
where
    F: Fn() + Send + Sync + 'static,
{
    {
        let mut cb_guard = CALLBACK
            .lock()
            .map_err(|_| anyhow!("Callback mutex poisoned"))?;
        *cb_guard = Some(Arc::new(callback));
    }

    let (stop_tx, stop_rx) = mpsc::channel();
    let (ready_tx, ready_rx) = mpsc::channel();

    let thread = thread::spawn(move || unsafe {
        let hook = match SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), HINSTANCE::default(), 0) {
            Ok(hook) => {
                let _ = ready_tx.send(Ok(()));
                hook
            }
            Err(err) => {
                let _ = ready_tx.send(Err(anyhow!("Failed to install mouse hook: {err:?}")));
                return;
            }
        };

        log::info!("[GlobalLookup] CTRL+RightClick hook installed");

        let mut msg = MSG::default();
        loop {
            // Exit when stop signal is received
            if matches!(
                stop_rx.try_recv(),
                Ok(_) | Err(mpsc::TryRecvError::Disconnected)
            ) {
                break;
            }

            // Wait for messages or timeout (100ms) to check stop signal
            // Using MsgWaitForMultipleObjects is more efficient than sleep+poll
            MsgWaitForMultipleObjects(Some(&[] as &[HANDLE]), false, 100, QS_ALLINPUT);

            // Pump Windows messages so the low-level hook receives events
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).into() {
                if msg.message == WM_QUIT {
                    break;
                }
                let _ = TranslateMessage(&msg);
                let _ = DispatchMessageW(&msg);
            }
        }

        if let Err(err) = UnhookWindowsHookEx(hook) {
            log::warn!("[GlobalLookup] Failed to remove mouse hook: {err:?}");
        } else {
            log::info!("[GlobalLookup] CTRL+RightClick hook removed");
        }
    });

    match ready_rx
        .recv()
        .map_err(|_| anyhow!("Mouse hook thread failed to start"))?
    {
        Ok(()) => Ok(CtrlRightClickHook {
            stop_tx,
            thread: Some(thread),
        }),
        Err(err) => {
            let _ = stop_tx.send(());
            let _ = thread.join();
            Err(err)
        }
    }
}

unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let message = wparam.0 as u32;
        if (message == WM_RBUTTONDOWN || message == WM_RBUTTONUP) && is_ctrl_down() {
            if message == WM_RBUTTONDOWN {
                log::debug!("[GlobalLookup] CTRL+RightClick detected");
                // Clone the callback Arc and release lock before invoking
                // This prevents holding the lock during potentially long callback execution
                let callback = CALLBACK.lock().ok().and_then(|guard| guard.clone());
                if let Some(cb) = callback {
                    cb();
                }
            }
            return LRESULT(1);
        }
    }

    CallNextHookEx(HHOOK::default(), code, wparam, lparam)
}

fn is_ctrl_down() -> bool {
    // VK_CONTROL covers both left and right Ctrl keys
    unsafe { (GetAsyncKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0 }
}
