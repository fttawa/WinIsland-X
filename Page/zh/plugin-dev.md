# 插件开发

WinIsland 拥有功能强大的基于 DLL 的插件系统，允许开发者扩展其功能、添加自定义小组件或实现系统监视器。

## 快速开始

开始开发最简单的方法是参考源码中的 `winisland_sample_plugin` 目录。

### 前提条件
- [Rust](https://www.rust-lang.org/)
- 了解基础的 FFI (外部函数接口) 知识

## 系统架构

插件被编译为动态链接库（Windows 上为 `.dll`）。WinIsland 会在启动时扫描 `plugins/` 目录并加载所有有效的模块。

### 核心接口

你的插件需要导出特定的 C 兼容函数。

#### 1. 插件信息
提供插件的元数据。

```rust
#[no_mangle]
pub unsafe extern "C" fn plugin_get_info() -> PluginInfo {
    PluginInfo {
        name: b"我的超酷插件\0".as_ptr() as *const c_char,
        version: b"1.0.0\0".as_ptr() as *const c_char,
        author: b"你\0".as_ptr() as *const c_char,
        description: b"实现了一些惊人的功能。\0".as_ptr() as *const c_char,
    }
}
```

#### 2. 初始化
接收用于与主程序交互的回调函数。

```rust
#[no_mangle]
pub unsafe extern "C" fn plugin_init(cb: *const PluginCallbacks) -> *mut c_void {
    // 存储回调以备后用
    std::ptr::null_mut() // 如果需要，返回实例指针
}
```

#### 3. 更新循环
每一帧都会运行的逻辑。

```rust
#[no_mangle]
pub unsafe extern "C" fn plugin_on_update(_instance: *mut c_void, ctx: *const PluginContext) {
    let context = &*ctx;
    // 检查 context.is_expanded 等状态
}
```

## 插件能力

插件可以使用 `PluginCallbacks` 执行以下操作：
- **设置自定义文本**: 使用 `set_custom_text` 在展开后的岛屿区域显示信息（如天气、网速）。
- **控制展开状态**: 请求岛屿展开或收起。
- **日志记录**: 向主程序控制台发送日志。
- **自定义 UI**: 通过 `plugin_open_config_ui` 提供原生的配置窗口。

## 部署

1. 将你的插件编译为 `cdylib`。
2. 将生成的 `.dll` 文件复制到 `WinIsland.exe` 同级目录下的 `plugins/` 文件夹中。
3. 在 **设置 -> 插件** 中启用它。
