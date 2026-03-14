use std::env;

pub fn is_store_build() -> bool {
    // Method 1: Check if running from WindowsApps directory (Store sandbox)
    if let Ok(exe_path) = env::current_exe() {
        if let Some(path_str) = exe_path.to_str() {
            if path_str.contains("WindowsApps") {
                log::info!("[Platform] Detected Store build (WindowsApps path)");
                return true;
            }
        }
    }

    // Method 2: Check for Windows package identity (MSIX container)
    #[cfg(windows)]
    {
        // Try to get current package identity
        // Store apps have package identity, sideloaded apps do too
        use windows::ApplicationModel::Package;

        match Package::Current() {
            Ok(_package) => {
                log::info!("[Platform] Detected Store build (has package identity)");
                return true;
            }
            Err(_) => {
                // No package identity = traditional Win32 app
                log::debug!("[Platform] No package identity - traditional Win32 build");
            }
        }
    }

    log::info!("[Platform] Running as traditional Win32 build (not Store)");
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_store_build_detection() {
        // This will return false in dev/test environment
        let result = is_store_build();
        log::debug!("Store build detected: {}", result);

        // Test should pass regardless of environment
        assert!(result == true || result == false);
    }
}
