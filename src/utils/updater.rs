use std::fs;
use std::path::{PathBuf};
use serde::{Deserialize, Serialize};
use crate::core::config::{AppConfig};
use std::process::Command;
use std::io::Read;
use windows::core::PCWSTR;
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OKCANCEL, MB_ICONINFORMATION, MB_TOPMOST, MB_SETFOREGROUND, IDOK, IDYES};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionInfo {
    pub timestamp: String,
}

const UPDATE_URL_JSON: &str = "https://github.com/Eatgrapes/WinIsland/releases/download/nightly/version_info.json";
const UPDATE_URL_EXE: &str = "https://github.com/Eatgrapes/WinIsland/releases/download/nightly/WinIsland.exe";

pub fn get_app_dir() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".winisland");
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }
    path
}

pub fn check_for_updates(config: &AppConfig) {
    if !config.check_for_updates {
        return;
    }

    std::thread::spawn(move || {
        let app_dir = get_app_dir();
        let local_json_path = app_dir.join("version_info.json");
        
        let remote_json_str = match ureq::get(UPDATE_URL_JSON).call() {
            Ok(resp) => match resp.into_string() {
                Ok(s) => s,
                Err(_) => return,
            },
            Err(_) => return,
        };

        let remote_info: VersionInfo = match serde_json::from_str(&remote_json_str) {
            Ok(info) => info,
            Err(_) => return,
        };

        let mut needs_update = false;
        if local_json_path.exists() {
            if let Ok(local_content) = fs::read_to_string(&local_json_path) {
                if let Ok(local_info) = serde_json::from_str::<VersionInfo>(&local_content) {
                    if remote_info.timestamp > local_info.timestamp {
                        needs_update = true;
                    }
                } else {
                    needs_update = true;
                }
            } else {
                needs_update = true;
            }
        } else {
            needs_update = true;
        }

        if needs_update {
            let title: Vec<u16> = "Update Available\0".encode_utf16().collect();
            let text: Vec<u16> = format!("A new version of WinIsland is available (Released: {}). Would you like to update now?\0", remote_info.timestamp).encode_utf16().collect();
            
            let result = unsafe {
                MessageBoxW(
                    None,
                    PCWSTR(text.as_ptr()),
                    PCWSTR(title.as_ptr()),
                    MB_OKCANCEL | MB_ICONINFORMATION | MB_TOPMOST | MB_SETFOREGROUND
                )
            };

            if result == IDOK || result == IDYES {
                perform_update(remote_json_str, app_dir);
            }
        }
    });
}

fn perform_update(remote_json_str: String, app_dir: PathBuf) {
    let resp = match ureq::get(UPDATE_URL_EXE).call() {
        Ok(r) => r,
        Err(_) => {
            let title: Vec<u16> = "Update Failed\0".encode_utf16().collect();
            let text: Vec<u16> = "Failed to download the new version.\0".encode_utf16().collect();
            unsafe {
                MessageBoxW(None, PCWSTR(text.as_ptr()), PCWSTR(title.as_ptr()), MB_ICONINFORMATION | MB_TOPMOST);
            }
            return;
        }
    };

    let mut bytes = Vec::new();
    if resp.into_reader().read_to_end(&mut bytes).is_err() {
        return;
    }

    let current_exe = std::env::current_exe().unwrap();
    let new_exe_path = current_exe.with_extension("exe.new");
    
    if fs::write(&new_exe_path, &bytes).is_err() {
        let title: Vec<u16> = "Update Failed\0".encode_utf16().collect();
        let text: Vec<u16> = "Failed to save the new version.\0".encode_utf16().collect();
        unsafe {
            MessageBoxW(None, PCWSTR(text.as_ptr()), PCWSTR(title.as_ptr()), MB_ICONINFORMATION | MB_TOPMOST);
        }
        return;
    }

    let local_json_path = app_dir.join("version_info.json");
    let _ = fs::write(local_json_path, remote_json_str);

    let current_exe_str = current_exe.to_str().unwrap();
    let new_exe_str = new_exe_path.to_str().unwrap();
    
    let pid = std::process::id();
    let script = format!(
        "Start-Sleep -Seconds 1; \
         while (Get-Process -Id {} -ErrorAction SilentlyContinue) {{ Start-Sleep -Milliseconds 100 }}; \
         Move-Item -Path '{}' -Destination '{}' -Force; \
         Start-Process -FilePath '{}'",
        pid, new_exe_str, current_exe_str, current_exe_str
    );

    let _ = Command::new("powershell")
        .args(["-WindowStyle", "Hidden", "-Command", &script])
        .spawn();

    std::process::exit(0);
}
