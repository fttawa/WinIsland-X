use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use once_cell::sync::Lazy;
use windows::Win32::Globalization::GetUserDefaultLocaleName;

pub struct I18n {
    pub current_lang: String,
    translations: HashMap<String, String>,
}

static I18N: Lazy<Arc<RwLock<I18n>>> = Lazy::new(|| {
    let mut i18n = I18n {
        current_lang: "en".to_string(),
        translations: HashMap::new(),
    };
    i18n.load("en");
    Arc::new(RwLock::new(i18n))
});

const LANG_EN: &str = r#"
tab_general=General
tab_appearance=Appearance
tab_performance=Performance
tab_plugins=Plugins
tab_about=About
plugin_name=Plugin Name
plugin_version=Version
plugin_author=Author
plugin_config=Config
plugin_enable=Enable
plugin_disable=Disable
no_plugins=No plugins loaded.
global_scale=Global Scale
base_width=Base Width
base_height=Base Height
expanded_width=Expanded Width
expanded_height=Expanded Height
adaptive_border=Adaptive Border
motion_blur=Motion Blur
custom_font=Custom Font
font_select=Select
font_reset=Reset
start_boot=Start at Boot
auto_hide=Auto Hide
check_updates=Check for Updates
update_interval=Update Interval (h)
language=Language
lang_name=English
hide_delay=Hide Delay (s)
reset_defaults=Reset to Defaults
visit_homepage=Visit Project Homepage
created_by=Created by
window_effect=Window Effect
effect_none=None
effect_acrylic=Acrylic
effect_mica=Mica
effect_liquid=Liquid Glass
theme_colors=Theme Colors
custom_theme_color=Custom Theme Color
is_solid_theme=Solid Theme Color
show_progress_bar=Show Progress Bar
progress_bar_style=Progress Bar Style
style_gradient=Gradient
style_solid=Solid
smooth_transition=Smooth Transition
show_fps=Show FPS
show_gpu_status=Show GPU Status
fps_limit=Frame Rate Limit
fps_unlimited=Unlimited
use_gpu=GPU Acceleration
music_settings_title=Music Settings
smtc_control=SMTC Control
show_lyrics=Show Lyrics
lyrics_source=Lyrics Source
lyrics_fallback=Fallback Source
media_apps=MEDIA APPLICATIONS
scan_apps=Scan Apps
no_sessions=No sessions detected
delete=Delete
update_available_title=Update Available
update_available_desc=A new version of WinIsland is available (Released: {}). Would you like to update now?
update_failed_title=Update Failed
update_failed_dl=Failed to download the new version.
update_failed_save=Failed to save the new version.
"#;

const LANG_ZH: &str = r#"
tab_general=常规
tab_appearance=外观
tab_performance=性能
tab_plugins=插件
tab_about=关于
plugin_name=插件名称
plugin_version=版本
plugin_author=作者
plugin_config=设置
plugin_enable=启用
plugin_disable=禁用
no_plugins=暂无已加载的插件。
global_scale=全局缩放
base_width=基础宽度
base_height=基础高度
expanded_width=展开宽度
expanded_height=展开高度
adaptive_border=自适应边框
motion_blur=动态模糊
custom_font=自定义字体
font_select=选择
font_reset=重置
start_boot=开机启动
auto_hide=自动隐藏
check_updates=检查更新
update_interval=检查更新间隔 (h)
language=语言
lang_name=中文
hide_delay=隐藏延迟 (s)
reset_defaults=恢复默认设置
visit_homepage=访问项目主页
created_by=作者
window_effect=窗口效果
effect_none=无
effect_acrylic=亚克力
effect_mica=Mica (云母)
effect_liquid=液态玻璃
theme_colors=主题颜色
custom_theme_color=自定义主题颜色
is_solid_theme=纯色主题
show_progress_bar=显示进度条
progress_bar_style=进度条样式
style_gradient=渐变
style_solid=纯色
smooth_transition=平滑过渡
show_fps=显示帧率 (FPS)
show_gpu_status=显示 GPU 状态
fps_limit=帧率限制
fps_unlimited=无限制
use_gpu=GPU 硬件加速
music_settings_title=音乐设置
smtc_control=SMTC 控制
show_lyrics=歌词显示
lyrics_source=歌词来源
lyrics_fallback=备选来源
media_apps=媒体应用程序
scan_apps=扫描应用
no_sessions=未检测到运行中的媒体
delete=删除
update_available_title=发现新版本
update_available_desc=WinIsland 有新版本可用 (发布时间: {})。是否现在更新？
update_failed_title=更新失败
update_failed_dl=无法下载新版本。
update_failed_save=无法保存新版本文件。
"#;

impl I18n {
    pub fn load(&mut self, lang: &str) {
        let content = match lang {
            "zh" => LANG_ZH,
            _ => LANG_EN,
        };
        self.current_lang = lang.to_string();
        self.translations.clear();
        for line in content.lines() {
            if let Some((k, v)) = line.split_once('=') {
                self.translations.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
    }

    pub fn get(&self, key: &str) -> String {
        self.translations.get(key).cloned().unwrap_or_else(|| key.to_string())
    }
}

pub fn init_i18n(config_lang: &str) {
    let mut target_lang = config_lang.to_string();
    if target_lang == "auto" {
        target_lang = get_system_lang();
    }
    I18N.write().unwrap().load(&target_lang);
}

pub fn set_lang(lang: &str) {
    I18N.write().unwrap().load(lang);
}

pub fn current_lang() -> String {
    I18N.read().unwrap().current_lang.clone()
}

pub fn tr(key: &str) -> String {
    I18N.read().unwrap().get(key)
}

fn get_system_lang() -> String {
    let mut buffer = [0u16; 128];
    unsafe {
        let len = GetUserDefaultLocaleName(&mut buffer);
        if len > 0 {
            let s = String::from_utf16_lossy(&buffer[..len as usize - 1]);
            if s.starts_with("zh") {
                return "zh".to_string();
            }
        }
    }
    "en".to_string()
}
