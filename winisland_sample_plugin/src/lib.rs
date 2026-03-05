use std::os::raw::{c_char, c_void};
use std::ffi::CString;

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

static mut CALLBACKS: Option<PluginCallbacks> = None;

#[no_mangle]
pub unsafe extern "C" fn plugin_get_info() -> PluginInfo {
    PluginInfo {
        name: b"Hello World Plugin\0".as_ptr() as *const c_char,
        version: b"1.0.0\0".as_ptr() as *const c_char,
        author: b"Developer\0".as_ptr() as *const c_char,
        description: b"A sample plugin that displays text on the island and can auto-expand.\0".as_ptr() as *const c_char,
    }
}

#[no_mangle]
pub unsafe extern "C" fn plugin_init(cb: *const PluginCallbacks) -> *mut c_void {
    CALLBACKS = Some(std::ptr::read(cb));
    
    if let Some(cbs) = &CALLBACKS {
        let msg = CString::new("Hello World Plugin Initialized").unwrap();
        (cbs.log_msg)(msg.as_ptr());
    }
    
    std::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn plugin_on_update(_instance: *mut c_void, ctx: *const PluginContext) {
    let context = &*ctx;
    if let Some(cbs) = &CALLBACKS {
        if context.is_expanded {
            let text = format!("Time: {:.1}s | Custom Plugin Active", context.app_time);
            let c_text = CString::new(text).unwrap();
            (cbs.set_custom_text)(c_text.as_ptr());
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn plugin_has_config_ui() -> bool {
    true
}

#[no_mangle]
pub unsafe extern "C" fn plugin_open_config_ui() {
    use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK, MB_ICONINFORMATION};
    use windows::core::w;
    use windows::Win32::Foundation::HWND;
    
    unsafe {
        MessageBoxW(
            HWND(std::ptr::null_mut()), 
            w!("This is a custom settings UI provided by the plugin itself. Here you can configure plugin behaviors via system APIs."), 
            w!("Plugin Configuration"), 
            MB_OK | MB_ICONINFORMATION
        );
    }
}