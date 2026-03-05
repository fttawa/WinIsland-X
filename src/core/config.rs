use serde::{Deserialize, Serialize};
pub const APP_VERSION: &str = "1.0.0";
pub const APP_AUTHOR: &str = "Eatgrapes";
pub const APP_HOMEPAGE: &str = "https://github.com/Eatgrapes/WinIsland";
pub const WINDOW_TITLE: &str = "WinIsland";
pub const TOP_OFFSET: i32 = 10;
pub const PADDING: f32 = 80.0;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum WindowEffect {
    None,
    Acrylic,
    Mica,
    LiquidGlass,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ThemeColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
    pub position: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ProgressBarStyle {
    Gradient,
    Solid,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AppConfig {
    pub global_scale: f32,
    pub base_width: f32,
    pub base_height: f32,
    pub expanded_width: f32,
    pub expanded_height: f32,
    pub adaptive_border: bool,
    pub motion_blur: bool,
    pub smtc_enabled: bool,
    pub smtc_apps: Vec<String>,
    #[serde(default = "default_show_lyrics")]
    pub show_lyrics: bool,
    #[serde(default = "default_custom_font")]
    pub custom_font_path: Option<String>,
    #[serde(default = "default_auto_start")]
    pub auto_start: bool,
    #[serde(default = "default_auto_hide")]
    pub auto_hide: bool,
    #[serde(default = "default_auto_hide_delay")]
    pub auto_hide_delay: f32,
    #[serde(default = "default_check_for_updates")]
    pub check_for_updates: bool,
    #[serde(default = "default_update_check_interval")]
    pub update_check_interval: f32,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_lyrics_source")]
    pub lyrics_source: String,
    #[serde(default = "default_lyrics_fallback")]
    pub lyrics_fallback: bool,
    // New Performance & Appearance fields
    #[serde(default = "default_fps_limit")]
    pub fps_limit: i32,
    #[serde(default = "default_use_gpu")]
    pub use_gpu: bool,
    #[serde(default = "default_window_effect")]
    pub window_effect: WindowEffect,
    #[serde(default = "default_theme_colors")]
    pub theme_colors: Vec<ThemeColor>,
    #[serde(default = "default_smooth_transition")]
    pub smooth_transition: bool,
    #[serde(default = "default_show_progress_bar")]
    pub show_progress_bar: bool,
    #[serde(default = "default_progress_bar_style")]
    pub progress_bar_style: ProgressBarStyle,
    #[serde(default = "default_is_solid_theme")]
    pub is_solid_theme: bool,
    #[serde(default = "default_show_fps")]
    pub show_fps: bool,
    #[serde(default = "default_show_gpu_status")]
    pub show_gpu_status: bool,
}

fn default_show_lyrics() -> bool { true }
fn default_custom_font() -> Option<String> { None }
fn default_auto_start() -> bool { false }
fn default_auto_hide() -> bool { false }
fn default_auto_hide_delay() -> f32 { 5.0 }
fn default_check_for_updates() -> bool { true }
fn default_update_check_interval() -> f32 { 4.0 }
fn default_language() -> String { "auto".to_string() }
fn default_lyrics_source() -> String { "163".to_string() }
fn default_lyrics_fallback() -> bool { true }
fn default_fps_limit() -> i32 { 60 }
fn default_use_gpu() -> bool { true }
fn default_window_effect() -> WindowEffect { WindowEffect::None }
fn default_smooth_transition() -> bool { true }
fn default_show_progress_bar() -> bool { true }
fn default_progress_bar_style() -> ProgressBarStyle { ProgressBarStyle::Gradient }
fn default_is_solid_theme() -> bool { false }
fn default_show_fps() -> bool { true }
fn default_show_gpu_status() -> bool { true }
fn default_theme_colors() -> Vec<ThemeColor> {
    vec![
        ThemeColor { r: 0, g: 0, b: 0, a: 255, position: 0.0 },
        ThemeColor { r: 0, g: 0, b: 0, a: 255, position: 1.0 },
    ]
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            global_scale: 1.0,
            base_width: 120.0,
            base_height: 27.0,
            expanded_width: 360.0,
            expanded_height: 190.0,
            adaptive_border: false,
            motion_blur: true,
            smtc_enabled: true,
            smtc_apps: Vec::new(),
            show_lyrics: true,
            custom_font_path: None,
            auto_start: false,
            auto_hide: false,
            auto_hide_delay: 5.0,
            check_for_updates: true,
            update_check_interval: 4.0,
            language: "auto".to_string(),
            lyrics_source: "163".to_string(),
            lyrics_fallback: true,
            fps_limit: 60,
            use_gpu: true,
            window_effect: WindowEffect::None,
            theme_colors: default_theme_colors(),
            smooth_transition: true,
            show_progress_bar: true,
            progress_bar_style: ProgressBarStyle::Gradient,
            is_solid_theme: false,
            show_fps: true,
            show_gpu_status: true,
        }
    }
}
