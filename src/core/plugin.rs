use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

#[repr(C)]
pub struct PluginInfo {
    pub name: *const c_char,
    pub version: *const c_char,
    pub author: *const c_char,
    pub description: *const c_char,
}

#[repr(C)]
pub struct PluginContext {
    pub app_time: f32,
    pub is_expanded: bool,
    pub is_music_active: bool,
    pub current_w: f32,
    pub current_h: f32,
}

#[repr(C)]
pub struct PluginCallbacks {
    pub request_expand: extern "C" fn(),
    pub request_collapse: extern "C" fn(),
    pub log_msg: extern "C" fn(*const c_char),
    pub set_custom_text: extern "C" fn(*const c_char),
}

// Function signatures expected from the DLL
pub type PluginInitFn = unsafe extern "C" fn(*const PluginCallbacks) -> *mut c_void;
pub type PluginGetInfoFn = unsafe extern "C" fn() -> PluginInfo;
pub type PluginOnUpdateFn = unsafe extern "C" fn(*mut c_void, *const PluginContext);
pub type PluginHasConfigUiFn = unsafe extern "C" fn() -> bool;
pub type PluginOpenConfigUiFn = unsafe extern "C" fn();

pub struct LoadedPlugin {
    pub id: String,
    pub path: PathBuf,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub enabled: bool,
    library: Library,
    instance: *mut c_void,
    pub on_update_fn: Option<PluginOnUpdateFn>,
    pub has_config_ui_fn: Option<PluginHasConfigUiFn>,
    pub open_config_ui_fn: Option<PluginOpenConfigUiFn>,
}

unsafe impl Send for LoadedPlugin {}
unsafe impl Sync for LoadedPlugin {}

pub struct PluginManager {
    pub plugins: HashMap<String, LoadedPlugin>,
    pub expand_requested: bool,
    pub collapse_requested: bool,
    pub custom_text: String,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            expand_requested: false,
            collapse_requested: false,
            custom_text: String::new(),
        }
    }

    pub fn load_plugin(&mut self, path: PathBuf) -> Result<(), String> {
        unsafe {
            let lib = Library::new(&path).map_err(|e| e.to_string())?;
            
            let get_info: Symbol<PluginGetInfoFn> = lib.get(b"plugin_get_info\0").map_err(|e| e.to_string())?;
            let info = get_info();
            
            let name = CStr::from_ptr(info.name).to_string_lossy().into_owned();
            let version = CStr::from_ptr(info.version).to_string_lossy().into_owned();
            let author = CStr::from_ptr(info.author).to_string_lossy().into_owned();
            let description = CStr::from_ptr(info.description).to_string_lossy().into_owned();
            
            let init: Symbol<PluginInitFn> = lib.get(b"plugin_init\0").map_err(|e| e.to_string())?;
            
            let callbacks = PluginCallbacks {
                request_expand: plugin_cb_request_expand,
                request_collapse: plugin_cb_request_collapse,
                log_msg: plugin_cb_log_msg,
                set_custom_text: plugin_cb_set_custom_text,
            };
            
            let instance = init(&callbacks);
            
            let on_update_fn = lib.get::<PluginOnUpdateFn>(b"plugin_on_update\0").ok().map(|s| *s);
            let has_config_ui_fn = lib.get::<PluginHasConfigUiFn>(b"plugin_has_config_ui\0").ok().map(|s| *s);
            let open_config_ui_fn = lib.get::<PluginOpenConfigUiFn>(b"plugin_open_config_ui\0").ok().map(|s| *s);
            
            let id = name.clone();
            
            let plugin = LoadedPlugin {
                id: id.clone(),
                path,
                name,
                version,
                author,
                description,
                enabled: true,
                library: lib,
                instance,
                on_update_fn,
                has_config_ui_fn,
                open_config_ui_fn,
            };
            
            self.plugins.insert(id, plugin);
            Ok(())
        }
    }

    pub fn scan_plugins(&mut self) {
        let exe_path = std::env::current_exe().unwrap_or_default();
        let plugin_dir = exe_path.parent().unwrap_or(std::path::Path::new("")).join("plugins");
        
        if !plugin_dir.exists() {
            let _ = std::fs::create_dir_all(&plugin_dir);
            return;
        }

        if let Ok(entries) = std::fs::read_dir(plugin_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some(std::env::consts::DLL_EXTENSION) {
                    let _ = self.load_plugin(path);
                }
            }
        }
    }

    pub fn update(&mut self, ctx: &PluginContext) {
        for plugin in self.plugins.values_mut() {
            if plugin.enabled {
                if let Some(update_fn) = plugin.on_update_fn {
                    unsafe { update_fn(plugin.instance, ctx) };
                }
            }
        }
    }
}

pub static PLUGIN_MANAGER: Lazy<Arc<Mutex<PluginManager>>> = Lazy::new(|| {
    Arc::new(Mutex::new(PluginManager::new()))
});

// Callbacks
extern "C" fn plugin_cb_request_expand() {
    if let Ok(mut pm) = PLUGIN_MANAGER.lock() {
        pm.expand_requested = true;
    }
}

extern "C" fn plugin_cb_request_collapse() {
    if let Ok(mut pm) = PLUGIN_MANAGER.lock() {
        pm.collapse_requested = true;
    }
}

extern "C" fn plugin_cb_log_msg(msg: *const c_char) {
    unsafe {
        if !msg.is_null() {
            let s = CStr::from_ptr(msg).to_string_lossy();
            println!("[Plugin] {}", s);
        }
    }
}

extern "C" fn plugin_cb_set_custom_text(msg: *const c_char) {
    unsafe {
        if !msg.is_null() {
            let s = CStr::from_ptr(msg).to_string_lossy().into_owned();
            if let Ok(mut pm) = PLUGIN_MANAGER.lock() {
                pm.custom_text = s;
            }
        } else {
            if let Ok(mut pm) = PLUGIN_MANAGER.lock() {
                pm.custom_text.clear();
            }
        }
    }
}
