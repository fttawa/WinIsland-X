# Plugin Development

WinIsland features a powerful DLL-based plugin system that allows developers to extend its functionality, add custom widgets, or implement system monitors.

## Quick Start

The easiest way to start is by checking the `winisland_sample_plugin` directory in the source code.

### Prerequisites
- [Rust](https://www.rust-lang.org/)
- Basic knowledge of FFI (Foreign Function Interface)

## Architecture

Plugins are compiled as dynamic libraries (`.dll` on Windows). WinIsland scans the `plugins/` directory at startup and loads all valid modules.

### Core Interface

Your plugin should export specific C-compatible functions.

#### 1. Plugin Information
Provide metadata about your plugin.

```rust
#[no_mangle]
pub unsafe extern "C" fn plugin_get_info() -> PluginInfo {
    PluginInfo {
        name: b"My Awesome Plugin\0".as_ptr() as *const c_char,
        version: b"1.0.0\0".as_ptr() as *const c_char,
        author: b"You\0".as_ptr() as *const c_char,
        description: b"Does something amazing.\0".as_ptr() as *const c_char,
    }
}
```

#### 2. Initialization
Receive callbacks to interact with the host.

```rust
#[no_mangle]
pub unsafe extern "C" fn plugin_init(cb: *const PluginCallbacks) -> *mut c_void {
    // Store callbacks for later use
    std::ptr::null_mut() // Return an instance pointer if needed
}
```

#### 3. Update Loop
Logic that runs every frame.

```rust
#[no_mangle]
pub unsafe extern "C" fn plugin_on_update(_instance: *mut c_void, ctx: *const PluginContext) {
    let context = &*ctx;
    // Check context.is_expanded, etc.
}
```

## Capabilities

Plugins can use `PluginCallbacks` to:
- **Set Custom Text**: Display information in the expanded island area using `set_custom_text`.
- **Control Expansion**: Request the island to expand or collapse.
- **Logging**: Send logs to the main application console.
- **Custom UI**: Provide a native configuration window via `plugin_open_config_ui`.

## Deployment

1. Build your plugin as a `cdylib`.
2. Copy the resulting `.dll` to the `plugins/` folder next to `WinIsland.exe`.
3. Enable it in **Settings -> Plugins**.
