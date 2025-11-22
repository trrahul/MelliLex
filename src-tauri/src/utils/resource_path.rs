use once_cell::sync::OnceCell;
use std::path::PathBuf;

static RESOURCE_DIR: OnceCell<PathBuf> = OnceCell::new();

pub fn init(resource_dir: PathBuf) {
    let _ = RESOURCE_DIR.set(resource_dir);
}

pub fn resolve(relative_path: &str) -> PathBuf {
    // Try the initialized resource dir first
    if let Some(base) = RESOURCE_DIR.get() {
        let path = base.join(relative_path);
        if path.exists() {
            return path;
        }
    }

    // For bundled apps, resources are in the same directory as the executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // Resources are bundled in <exe_dir>/resources/
            let bundled_path = exe_dir.join("resources").join(relative_path);
            if bundled_path.exists() {
                return bundled_path;
            }
        }
    }

    // Fallback for development: try relative path from current directory
    let relative = PathBuf::from("resources").join(relative_path);
    if relative.exists() {
        return relative;
    }

    // Try from src-tauri directory (for tests and dev)
    let from_src_tauri = PathBuf::from("src-tauri").join("resources").join(relative_path);
    if from_src_tauri.exists() {
        return from_src_tauri;
    }

    // Return the exe-relative path even if it doesn't exist
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            return exe_dir.join("resources").join(relative_path);
        }
    }

    PathBuf::from("resources").join(relative_path)
}
