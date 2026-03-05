# WinIsland Development Guide

Welcome to the development guide for WinIsland. This document explains the architecture, the rendering pipeline, and how to extend the application using the Plugin System.

## 🏗 Architecture Overview

WinIsland is structured into several core modules:
- **`src/main.rs`**: Application entry point, handles single-instance check and initialization.
- **`src/window/app.rs`**: The main UI loop. Manages the window, event loop, and physics-based animations (Springs).
- **`src/core/render.rs`**: The rendering engine. Uses Skia to draw the island, effects (Acrylic/Liquid), and orchestrates component drawing.
- **`src/core/plugin.rs`**: The Plugin Manager. Dynamically loads DLLs and handles FFI communication.
- **`src/ui/expanded/`**: Components for the expanded view (Music, Tools, etc.).

## 🔌 Plugin Development

WinIsland plugins are dynamic libraries (`.dll`) that implement a specific C-ABI interface.

### 1. Plugin Data Structures (FFI)

Plugins should use `#[repr(C)]` to ensure compatibility with the host.

```rust
#[repr(C)]
pub struct PluginInfo {
    pub name: *const c_char,
    pub version: *const c_char,
    pub author: *const c_char,
    pub description: *const c_char,
}

#[repr(C)]
pub struct PluginContext {
    pub app_time: f32,        // Seconds since app start
    pub is_expanded: bool,    // Current island state
    pub is_music_active: bool,
    pub current_w: f32,       // Current pixel width
    pub current_h: f32,       // Current pixel height
}

#[repr(C)]
pub struct PluginCallbacks {
    pub request_expand: extern "C" fn(),
    pub request_collapse: extern "C" fn(),
    pub log_msg: extern "C" fn(*const c_char),
    pub set_custom_text: extern "C" fn(*const c_char), // Display info in the island
}
```

### 2. Required Exports

Your DLL must export the following symbols:

- `plugin_get_info`: Returns basic metadata.
- `plugin_init`: Called once when the plugin is loaded. Provides the `PluginCallbacks`.
- `plugin_on_update`: Called every frame. Use this for logic or triggering UI changes.
- `plugin_has_config_ui` (Optional): Return `true` if you have a custom settings window.
- `plugin_open_config_ui` (Optional): Called when user clicks "Config" in Settings.

### 3. Example Plugin (Rust)

```rust
#[no_mangle]
pub unsafe extern "C" fn plugin_on_update(_instance: *mut c_void, ctx: *const PluginContext) {
    let context = &*ctx;
    if context.is_expanded {
        // Send custom text to the "System Status" area
        let text = CString::new("Hello from Plugin!").unwrap();
        (CALLBACKS.set_custom_text)(text.as_ptr());
    }
}
```

Refer to `winisland_sample_plugin/` in the repository for a boilerplate project.

## 🎨 Rendering Pipeline

The rendering is handled by **Skia**. Key concepts:
- **Physics**: We use `Spring` values for width, height, and radius. Never use linear interpolation for Dynamic Island animations.
- **Layers**:
    1. **Outer Glow**: A blurred drop shadow drawn outside the main rect.
    2. **Background**: Either a solid color, linear gradient, or a `RuntimeEffect` (Shader).
    3. **Acrylic Grain**: A noise shader applied on top of the background.
    4. **Rim Highlight**: A 1px stroke on the top edge to simulate physical light.
- **Scaling**: All dimensions are multiplied by `global_scale`. Ensure your components handle this.

## 🛠 Building for Development

To build with full debug symbols and faster compilation:
```bash
cargo build
```

To build the sample plugin:
```bash
cd winisland_sample_plugin
cargo build --release
# Copy the dll to the main project
copy target\release\winisland_sample_plugin.dll ..\target\debug\plugins\
```

## 🌍 Internationalization (i18n)

Translations are stored in `src/core/i18n.rs`. If you add a new UI element:
1. Add a key to `LANG_EN` and `LANG_ZH`.
2. Use `tr("your_key")` in the UI code.

## 🧪 Testing

We recommend testing plugin stability by running the app in a debugger. Since plugins are hot-loaded, ensure you handle thread safety if your plugin spawns its own background threads.
