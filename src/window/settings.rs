use crate::core::config::{AppConfig, APP_HOMEPAGE, APP_VERSION, APP_AUTHOR, WindowEffect, ThemeColor, ProgressBarStyle};
use crate::core::persistence::save_config;
use crate::core::i18n::{tr, set_lang, current_lang};
use crate::utils::color::*;
use skia_safe::{surfaces, Color, Font, FontMgr, FontStyle, Paint, Rect};
use softbuffer::{Context, Surface};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId, WindowButtons};
use crate::utils::autostart::set_autostart;

const SETTINGS_W: f32 = 400.0;
const SETTINGS_H: f32 = 550.0;
use crate::utils::icon::get_app_icon;

const THEME_PRESETS: &[(&str, [u8; 4], [u8; 4])] = &[
    ("Azure", [0, 120, 215, 255], [76, 194, 255, 255]),
    ("Sunset", [255, 82, 82, 255], [255, 171, 145, 255]),
    ("Forest", [0, 180, 70, 255], [130, 255, 100, 255]),
    ("Royal", [101, 31, 255, 255], [224, 64, 251, 255]),
    ("Monochrome", [66, 66, 66, 255], [180, 180, 180, 255]),
];

pub struct SettingsApp {
    window: Option<Arc<Window>>,
    surface: Option<Surface<Arc<Window>, Arc<Window>>>,
    sk_surface: Option<skia_safe::Surface>,
    config: AppConfig,
    active_tab: usize,
    border_switch_pos: f32,
    blur_switch_pos: f32,
    autostart_switch_pos: f32,
    update_switch_pos: f32,
    gpu_switch_pos: f32,
    smooth_switch_pos: f32,
    progress_switch_pos: f32,
    solid_theme_switch_pos: f32,
    fps_switch_pos: f32,
    gpu_status_switch_pos: f32,
    logical_mouse_pos: (f32, f32),
    font_mgr: FontMgr,
    custom_font_typeface: Option<skia_safe::Typeface>,
    custom_font_path_cache: Option<String>,
    frame_count: u64,
    scroll_y: f32,
    target_scroll_y: f32,
}

impl SettingsApp {
    pub fn new(config: AppConfig) -> Self {
        let initial_border = if config.adaptive_border { 1.0 } else { 0.0 };
        let initial_blur = if config.motion_blur { 1.0 } else { 0.0 };
        let initial_autostart = if config.auto_start { 1.0 } else { 0.0 };
        let initial_update = if config.check_for_updates { 1.0 } else { 0.0 };
        let initial_gpu = if config.use_gpu { 1.0 } else { 0.0 };
        let initial_smooth = if config.smooth_transition { 1.0 } else { 0.0 };
        let initial_progress = if config.show_progress_bar { 1.0 } else { 0.0 };
        let initial_solid = if config.is_solid_theme { 1.0 } else { 0.0 };
        let initial_fps_display = if config.show_fps { 1.0 } else { 0.0 };
        let initial_gpu_status = if config.show_gpu_status { 1.0 } else { 0.0 };
        let mut app = Self {
            window: None, surface: None, sk_surface: None, config, active_tab: 0,
            border_switch_pos: initial_border, blur_switch_pos: initial_blur, autostart_switch_pos: initial_autostart,
            update_switch_pos: initial_update, gpu_switch_pos: initial_gpu, smooth_switch_pos: initial_smooth,
            progress_switch_pos: initial_progress, solid_theme_switch_pos: initial_solid,
            fps_switch_pos: initial_fps_display, gpu_status_switch_pos: initial_gpu_status,
            logical_mouse_pos: (0.0, 0.0), font_mgr: FontMgr::new(), custom_font_typeface: None, custom_font_path_cache: None,
            frame_count: 0, scroll_y: 0.0, target_scroll_y: 0.0,
        };
        app.refresh_custom_font_cache();
        app
    }

    fn refresh_custom_font_cache(&mut self) {
        if self.custom_font_path_cache == self.config.custom_font_path { return; }
        self.custom_font_path_cache = self.config.custom_font_path.clone();
        self.custom_font_typeface = self.config.custom_font_path.as_ref().and_then(|path| std::fs::read(path).ok()).and_then(|data| self.font_mgr.new_from_data(&data, None));
    }

    fn get_font(&self, size: f32, bold: bool) -> Font {
        let style = if bold { FontStyle::bold() } else { FontStyle::normal() };
        if let Some(typeface) = &self.custom_font_typeface { return Font::from_typeface(typeface.clone(), size); }
        let typeface = self.font_mgr.match_family_style("Microsoft YaHei", style).or_else(|| self.font_mgr.match_family_style("Segoe UI", style)).unwrap_or_else(|| self.font_mgr.legacy_make_typeface(None, style).unwrap());
        Font::from_typeface(typeface, size)
    }

    fn draw(&mut self) {
        let win = self.window.as_ref().unwrap();
        let size = win.inner_size();
        let p_w = size.width as i32; let p_h = size.height as i32;
        if p_w <= 0 || p_h <= 0 { return; }

        let mut sk_surface = if let Some(ref s) = self.sk_surface {
            if s.width() == p_w && s.height() == p_h { s.clone() }
            else { let new_s = surfaces::raster_n32_premul(skia_safe::ISize::new(p_w, p_h)).unwrap(); self.sk_surface = Some(new_s.clone()); new_s }
        } else { let new_s = surfaces::raster_n32_premul(skia_safe::ISize::new(p_w, p_h)).unwrap(); self.sk_surface = Some(new_s.clone()); new_s };

        let canvas = sk_surface.canvas();
        canvas.reset_matrix(); canvas.clear(COLOR_BG);
        let scale = win.scale_factor() as f32; canvas.scale((scale, scale));
        let dx = ((p_w as f32 / scale) - SETTINGS_W) / 2.0; let dy = ((p_h as f32 / scale) - SETTINGS_H) / 2.0;
        canvas.translate((dx, dy));
        
        self.draw_tabs(canvas);
        canvas.save();
        canvas.clip_rect(Rect::from_xywh(0.0, 70.0, SETTINGS_W, SETTINGS_H - 70.0), skia_safe::ClipOp::Intersect, true);
        canvas.translate((0.0, -self.scroll_y));
        match self.active_tab { 0 => self.draw_general(canvas), 1 => self.draw_appearance(canvas), 2 => self.draw_performance(canvas), 3 => self.draw_plugins(canvas), _ => self.draw_about(canvas) }
        canvas.restore();

        let content_h = self.get_content_height(); let view_h = SETTINGS_H - 70.0;
        if content_h > view_h {
            let bar_h = (view_h / content_h) * view_h; let bar_y = 70.0 + (self.scroll_y / (content_h - view_h)) * (view_h - bar_h);
            let mut p = Paint::default(); p.set_anti_alias(true); p.set_color(Color::from_argb(80, 255, 255, 255));
            canvas.draw_round_rect(Rect::from_xywh(SETTINGS_W - 6.0, bar_y, 4.0, bar_h), 2.0, 2.0, &p);
        }

        if let Some(surface) = self.surface.as_mut() {
            let mut buffer = surface.buffer_mut().unwrap();
            let info = skia_safe::ImageInfo::new(skia_safe::ISize::new(p_w, p_h), skia_safe::ColorType::BGRA8888, skia_safe::AlphaType::Premul, None);
            let dst_row_bytes = (p_w * 4) as usize; let u8_buffer: &mut [u8] = bytemuck::cast_slice_mut(&mut *buffer);
            let _ = sk_surface.read_pixels(&info, u8_buffer, dst_row_bytes, (0, 0));
            buffer.present().unwrap();
        }
    }

    fn get_content_height(&self) -> f32 { match self.active_tab { 0 => 550.0, 1 => 1050.0, 2 => 400.0, 3 => 600.0, _ => 400.0 } }

    fn draw_tabs(&self, canvas: &skia_safe::Canvas) {
        let font = self.get_font(12.0, true);
        let mut paint = Paint::default(); paint.set_anti_alias(true);
        let tabs = [tr("tab_general"), tr("tab_appearance"), tr("tab_performance"), tr("tab_plugins"), tr("tab_about")];
        let tab_w = 75.0; let total_w = tab_w * tabs.len() as f32; let start_x = (SETTINGS_W - total_w) / 2.0;
        paint.set_color(COLOR_CARD); canvas.draw_round_rect(Rect::from_xywh(start_x, 20.0, total_w, 36.0), 10.0, 10.0, &paint);
        for (i, label) in tabs.iter().enumerate() {
            let bx = start_x + (i as f32 * tab_w);
            if self.active_tab == i { paint.set_color(COLOR_CARD_HIGHLIGHT); canvas.draw_round_rect(Rect::from_xywh(bx + 3.0, 23.0, tab_w - 6.0, 30.0), 8.0, 8.0, &paint); paint.set_color(COLOR_TEXT_PRI); }
            else { paint.set_color(COLOR_TEXT_SEC); }
            let (_, rect) = font.measure_str(label, None); canvas.draw_str(label, (bx + (tab_w - rect.width()) / 2.0, 43.0), &font, &paint);
        }
    }

    fn draw_general(&self, canvas: &skia_safe::Canvas) {
        let font = self.get_font(14.0, false);
        let mut paint = Paint::default(); paint.set_anti_alias(true);
        let items = [(tr("global_scale"), format!("{:.2}", self.config.global_scale)), (tr("base_width"), self.config.base_width.to_string()), (tr("base_height"), self.config.base_height.to_string()), (tr("expanded_width"), self.config.expanded_width.to_string()), (tr("expanded_height"), self.config.expanded_height.to_string())];
        let start_y = 90.0;
        for (i, (label, val)) in items.iter().enumerate() {
            let y = start_y + (i as f32 * 50.0); paint.set_color(COLOR_CARD); canvas.draw_round_rect(Rect::from_xywh(20.0, y - 5.0, SETTINGS_W - 40.0, 42.0), 10.0, 10.0, &paint);
            paint.set_color(COLOR_TEXT_PRI); canvas.draw_str(&label, (35.0, y + 21.0), &font, &paint);
            self.draw_button(canvas, 270.0, y + 2.0, "-");
            paint.set_color(COLOR_TEXT_PRI); let (_, rect) = font.measure_str(&val, None); canvas.draw_str(&val, (325.0 - rect.width() / 2.0, y + 21.0), &font, &paint);
            self.draw_button(canvas, 345.0, y + 2.0, "+");
        }
        let sw_border_y = start_y + (items.len() as f32 * 50.0) + 10.0;
        paint.set_color(COLOR_CARD); canvas.draw_round_rect(Rect::from_xywh(20.0, sw_border_y - 5.0, SETTINGS_W - 40.0, 42.0), 10.0, 10.0, &paint);
        paint.set_color(COLOR_TEXT_PRI); canvas.draw_str(&tr("adaptive_border"), (35.0, sw_border_y + 21.0), &font, &paint);
        self.draw_switch(canvas, 326.0, sw_border_y + 3.0, self.border_switch_pos);
        let sw_blur_y = sw_border_y + 50.0;
        paint.set_color(COLOR_CARD); canvas.draw_round_rect(Rect::from_xywh(20.0, sw_blur_y - 5.0, SETTINGS_W - 40.0, 42.0), 10.0, 10.0, &paint);
        paint.set_color(COLOR_TEXT_PRI); canvas.draw_str(&tr("motion_blur"), (35.0, sw_blur_y + 21.0), &font, &paint);
        self.draw_switch(canvas, 326.0, sw_blur_y + 3.0, self.blur_switch_pos);
        let autostart_y = sw_blur_y + 50.0;
        paint.set_color(COLOR_CARD); canvas.draw_round_rect(Rect::from_xywh(20.0, autostart_y - 5.0, SETTINGS_W - 40.0, 42.0), 10.0, 10.0, &paint);
        paint.set_color(COLOR_TEXT_PRI); canvas.draw_str(&tr("start_boot"), (35.0, autostart_y + 21.0), &font, &paint);
        self.draw_switch(canvas, 326.0, autostart_y + 3.0, self.autostart_switch_pos);
        let update_y = autostart_y + 50.0;
        paint.set_color(COLOR_CARD); canvas.draw_round_rect(Rect::from_xywh(20.0, update_y - 5.0, SETTINGS_W - 40.0, 42.0), 10.0, 10.0, &paint);
        paint.set_color(COLOR_TEXT_PRI); canvas.draw_str(&tr("check_updates"), (35.0, update_y + 21.0), &font, &paint);
        self.draw_switch(canvas, 326.0, update_y + 3.0, self.update_switch_pos);
        let lang_y = update_y + 50.0;
        paint.set_color(COLOR_CARD); canvas.draw_round_rect(Rect::from_xywh(20.0, lang_y - 5.0, SETTINGS_W - 40.0, 42.0), 10.0, 10.0, &paint);
        paint.set_color(COLOR_TEXT_PRI); canvas.draw_str(&tr("language"), (35.0, lang_y + 21.0), &font, &paint);
        self.draw_text_button(canvas, 300.0, lang_y + 3.0, 75.0, 26.0, &tr("lang_name"));
        let reset_y = lang_y + 70.0; paint.set_color(COLOR_DANGER); let reset_str = tr("reset_defaults"); let (_, rect) = font.measure_str(&reset_str, None); canvas.draw_str(&reset_str, ((SETTINGS_W - rect.width()) / 2.0, reset_y), &font, &paint);
    }

    fn draw_appearance(&self, canvas: &skia_safe::Canvas) {
        let font = self.get_font(14.0, false);
        let mut paint = Paint::default(); paint.set_anti_alias(true);
        let start_y = 90.0;
        let effect_y = start_y;
        paint.set_color(COLOR_CARD); canvas.draw_round_rect(Rect::from_xywh(20.0, effect_y - 5.0, SETTINGS_W - 40.0, 42.0), 10.0, 10.0, &paint);
        paint.set_color(COLOR_TEXT_PRI); canvas.draw_str(&tr("window_effect"), (35.0, effect_y + 21.0), &font, &paint);
        let effect_label = match self.config.window_effect { WindowEffect::None => tr("effect_none"), WindowEffect::Acrylic => tr("effect_acrylic"), WindowEffect::Mica => tr("effect_mica"), WindowEffect::LiquidGlass => tr("effect_liquid") };
        self.draw_text_button(canvas, 270.0, effect_y + 3.0, 105.0, 26.0, &effect_label);

        let smooth_y = effect_y + 50.0;
        paint.set_color(COLOR_CARD); canvas.draw_round_rect(Rect::from_xywh(20.0, smooth_y - 5.0, SETTINGS_W - 40.0, 42.0), 10.0, 10.0, &paint);
        paint.set_color(COLOR_TEXT_PRI); canvas.draw_str(&tr("smooth_transition"), (35.0, smooth_y + 21.0), &font, &paint);
        self.draw_switch(canvas, 326.0, smooth_y + 3.0, self.smooth_switch_pos);

        let progress_y = smooth_y + 50.0;
        paint.set_color(COLOR_CARD); canvas.draw_round_rect(Rect::from_xywh(20.0, progress_y - 5.0, SETTINGS_W - 40.0, 42.0), 10.0, 10.0, &paint);
        paint.set_color(COLOR_TEXT_PRI); canvas.draw_str(&tr("show_progress_bar"), (35.0, progress_y + 21.0), &font, &paint);
        self.draw_switch(canvas, 326.0, progress_y + 3.0, self.progress_switch_pos);

        let p_style_y = progress_y + 50.0;
        paint.set_color(COLOR_CARD); canvas.draw_round_rect(Rect::from_xywh(20.0, p_style_y - 5.0, SETTINGS_W - 40.0, 42.0), 10.0, 10.0, &paint);
        paint.set_color(COLOR_TEXT_PRI); canvas.draw_str(&tr("progress_bar_style"), (35.0, p_style_y + 21.0), &font, &paint);
        let style_label = match self.config.progress_bar_style { ProgressBarStyle::Gradient => tr("style_gradient"), ProgressBarStyle::Solid => tr("style_solid") };
        self.draw_text_button(canvas, 270.0, p_style_y + 3.0, 105.0, 26.0, &style_label);

        let solid_t_y = p_style_y + 50.0;
        paint.set_color(COLOR_CARD); canvas.draw_round_rect(Rect::from_xywh(20.0, solid_t_y - 5.0, SETTINGS_W - 40.0, 42.0), 10.0, 10.0, &paint);
        paint.set_color(COLOR_TEXT_PRI); canvas.draw_str(&tr("is_solid_theme"), (35.0, solid_t_y + 21.0), &font, &paint);
        self.draw_switch(canvas, 326.0, solid_t_y + 3.0, self.solid_theme_switch_pos);

        let theme_y = solid_t_y + 50.0; paint.set_color(COLOR_TEXT_SEC); canvas.draw_str(&tr("theme_colors"), (35.0, theme_y + 15.0), &font, &paint);
        let preset_start_y = theme_y + 30.0;
        for (i, (name, c1, _)) in THEME_PRESETS.iter().enumerate() {
            let y = preset_start_y + (i as f32 * 45.0); paint.set_color(COLOR_CARD); canvas.draw_round_rect(Rect::from_xywh(20.0, y - 5.0, SETTINGS_W - 40.0, 40.0), 10.0, 10.0, &paint);
            paint.set_color(Color::from_argb(c1[3], c1[0], c1[1], c1[2])); canvas.draw_circle((45.0, y + 15.0), 12.0, &paint);
            paint.set_color(COLOR_TEXT_PRI); canvas.draw_str(name, (70.0, y + 20.0), &font, &paint);
            if self.config.theme_colors.len() >= 2 { let cur = &self.config.theme_colors[0]; if cur.r == c1[0] && cur.g == c1[1] && cur.b == c1[2] { paint.set_color(COLOR_ACCENT); canvas.draw_str("✓", (SETTINGS_W - 60.0, y + 21.0), &self.get_font(18.0, true), &paint); } }
        }
        let custom_y = preset_start_y + (THEME_PRESETS.len() as f32 * 45.0) + 10.0;
        self.draw_text_button(canvas, 20.0, custom_y, SETTINGS_W - 40.0, 36.0, "Add Random Custom Theme Color");
    }

    fn draw_performance(&self, canvas: &skia_safe::Canvas) {
        let font = self.get_font(14.0, false);
        let mut paint = Paint::default(); paint.set_anti_alias(true);
        let start_y = 90.0; let fps_y = start_y;
        paint.set_color(COLOR_CARD); canvas.draw_round_rect(Rect::from_xywh(20.0, fps_y - 5.0, SETTINGS_W - 40.0, 42.0), 10.0, 10.0, &paint);
        paint.set_color(COLOR_TEXT_PRI); canvas.draw_str(&tr("fps_limit"), (35.0, fps_y + 21.0), &font, &paint);
        let fps_str = if self.config.fps_limit == 0 { tr("fps_unlimited") } else { self.config.fps_limit.to_string() };
        self.draw_button(canvas, 270.0, fps_y + 2.0, "-"); 
        let (_, rect) = font.measure_str(&fps_str, None); 
        canvas.draw_str(&fps_str, (321.0 - rect.width() / 2.0, fps_y + 21.0), &font, &paint);
        self.draw_button(canvas, 345.0, fps_y + 2.0, "+");
        
        let gpu_y = fps_y + 50.0;
        paint.set_color(COLOR_CARD); canvas.draw_round_rect(Rect::from_xywh(20.0, gpu_y - 5.0, SETTINGS_W - 40.0, 42.0), 10.0, 10.0, &paint);
        paint.set_color(COLOR_TEXT_PRI); canvas.draw_str(&tr("use_gpu"), (35.0, gpu_y + 21.0), &font, &paint);
        self.draw_switch(canvas, 326.0, gpu_y + 3.0, self.gpu_switch_pos);

        let show_fps_y = gpu_y + 50.0;
        paint.set_color(COLOR_CARD); canvas.draw_round_rect(Rect::from_xywh(20.0, show_fps_y - 5.0, SETTINGS_W - 40.0, 42.0), 10.0, 10.0, &paint);
        paint.set_color(COLOR_TEXT_PRI); canvas.draw_str(&tr("show_fps"), (35.0, show_fps_y + 21.0), &font, &paint);
        self.draw_switch(canvas, 326.0, show_fps_y + 3.0, self.fps_switch_pos);

        let show_gpu_text_y = show_fps_y + 50.0;
        paint.set_color(COLOR_CARD); canvas.draw_round_rect(Rect::from_xywh(20.0, show_gpu_text_y - 5.0, SETTINGS_W - 40.0, 42.0), 10.0, 10.0, &paint);
        paint.set_color(COLOR_TEXT_PRI); canvas.draw_str(&tr("show_gpu_status"), (35.0, show_gpu_text_y + 21.0), &font, &paint);
        self.draw_switch(canvas, 326.0, show_gpu_text_y + 3.0, self.gpu_status_switch_pos);
    }

    fn draw_plugins(&self, canvas: &skia_safe::Canvas) {
        let font = self.get_font(14.0, false);
        let mut paint = Paint::default(); paint.set_anti_alias(true);
        let start_y = 90.0;
        
        if let Ok(pm) = crate::core::plugin::PLUGIN_MANAGER.try_lock() {
            if pm.plugins.is_empty() {
                paint.set_color(COLOR_TEXT_SEC);
                let text = tr("no_plugins");
                let (_, rect) = font.measure_str(&text, None);
                canvas.draw_str(&text, ((SETTINGS_W - rect.width()) / 2.0, start_y + 20.0), &font, &paint);
            } else {
                let mut current_y = start_y;
                for (i, plugin) in pm.plugins.values().enumerate() {
                    let y = current_y + (i as f32 * 100.0);
                    paint.set_color(COLOR_CARD);
                    canvas.draw_round_rect(Rect::from_xywh(20.0, y - 5.0, SETTINGS_W - 40.0, 90.0), 10.0, 10.0, &paint);
                    
                    paint.set_color(COLOR_TEXT_PRI);
                    canvas.draw_str(&plugin.name, (35.0, y + 20.0), &self.get_font(16.0, true), &paint);
                    
                    paint.set_color(COLOR_TEXT_SEC);
                    canvas.draw_str(&format!("{} {}", tr("plugin_version"), plugin.version), (35.0, y + 42.0), &self.get_font(12.0, false), &paint);
                    canvas.draw_str(&plugin.description, (35.0, y + 64.0), &self.get_font(12.0, false), &paint);
                    
                    if plugin.has_config_ui_fn.is_some() {
                        self.draw_text_button(canvas, SETTINGS_W - 100.0, y + 10.0, 70.0, 26.0, &tr("plugin_config"));
                    }
                    
                    let toggle_str = if plugin.enabled { tr("plugin_disable") } else { tr("plugin_enable") };
                    let btn_color = if plugin.enabled { COLOR_DANGER } else { COLOR_ACCENT };
                    let mut btn_paint = Paint::default(); btn_paint.set_anti_alias(true); btn_paint.set_color(btn_color);
                    canvas.draw_round_rect(Rect::from_xywh(SETTINGS_W - 100.0, y + 45.0, 70.0, 26.0), 13.0, 13.0, &btn_paint);
                    btn_paint.set_color(COLOR_TEXT_PRI);
                    let (_, rect) = self.get_font(12.0, true).measure_str(&toggle_str, None);
                    canvas.draw_str(&toggle_str, (SETTINGS_W - 100.0 + (70.0 - rect.width()) / 2.0, y + 45.0 + 18.0), &self.get_font(12.0, true), &btn_paint);
                }
            }
        }
    }

    fn draw_text_button(&self, canvas: &skia_safe::Canvas, x: f32, y: f32, w: f32, h: f32, label: &str) {
        let mut paint = Paint::default(); paint.set_anti_alias(true); paint.set_color(COLOR_CARD_HIGHLIGHT); canvas.draw_round_rect(Rect::from_xywh(x, y, w, h), h/2.0, h/2.0, &paint);
        paint.set_color(COLOR_TEXT_PRI); let (_, rect) = self.get_font(12.0, true).measure_str(label, None); canvas.draw_str(label, (x + (w - rect.width()) / 2.0, y + h/2.0 + 5.0), &self.get_font(12.0, true), &paint);
    }

    fn draw_button(&self, canvas: &skia_safe::Canvas, x: f32, y: f32, label: &str) {
        let mut paint = Paint::default(); paint.set_anti_alias(true); paint.set_color(COLOR_CARD_HIGHLIGHT); canvas.draw_round_rect(Rect::from_xywh(x, y, 28.0, 28.0), 14.0, 14.0, &paint);
        paint.set_color(COLOR_TEXT_PRI); canvas.draw_str(label, (x + (28.0 - self.get_font(20.0, false).measure_str(label, None).1.width()) / 2.0, y + 20.0), &self.get_font(20.0, false), &paint);
    }

    fn draw_switch(&self, canvas: &skia_safe::Canvas, x: f32, y: f32, pos: f32) {
        let mut paint = Paint::default(); paint.set_anti_alias(true); let color_off = COLOR_CARD_HIGHLIGHT; let color_on = COLOR_ACCENT;
        let r = color_off.r() as f32 + (color_on.r() as f32 - color_off.r() as f32) * pos; let g = color_off.g() as f32 + (color_on.g() as f32 - color_off.g() as f32) * pos; let b = color_off.b() as f32 + (color_on.b() as f32 - color_off.b() as f32) * pos;
        paint.set_color(Color::from_rgb(r as u8, g as u8, b as u8)); canvas.draw_round_rect(Rect::from_xywh(x, y, 48.0, 26.0), 13.0, 13.0, &paint);
        paint.set_color(Color::WHITE); canvas.draw_round_rect(Rect::from_xywh(x + 2.0 + (pos * 22.0), y + 2.0, 22.0, 22.0), 11.0, 11.0, &paint);
    }

    fn draw_about(&self, canvas: &skia_safe::Canvas) {
        let mut paint = Paint::default(); paint.set_anti_alias(true); paint.set_color(COLOR_TEXT_PRI); canvas.draw_str("WinIsland", ((SETTINGS_W - self.get_font(28.0, true).measure_str("WinIsland", None).1.width()) / 2.0, 160.0), &self.get_font(28.0, true), &paint);
        paint.set_color(COLOR_TEXT_SEC); let v_str = format!("Version {}", APP_VERSION); canvas.draw_str(&v_str, ((SETTINGS_W - self.get_font(14.0, false).measure_str(&v_str, None).1.width()) / 2.0, 195.0), &self.get_font(14.0, false), &paint);
        let a_str = format!("{} {}", tr("created_by"), APP_AUTHOR); canvas.draw_str(&a_str, ((SETTINGS_W - self.get_font(14.0, false).measure_str(&a_str, None).1.width()) / 2.0, 220.0), &self.get_font(14.0, false), &paint);
        paint.set_color(COLOR_ACCENT); canvas.draw_str(tr("visit_homepage"), ((SETTINGS_W - self.get_font(14.0, false).measure_str(tr("visit_homepage"), None).1.width()) / 2.0, 280.0), &self.get_font(14.0, false), &paint);
    }

    fn handle_click(&mut self) {
        let (mx, my) = self.logical_mouse_pos; let mut changed = false; let win = self.window.as_ref().unwrap(); let scale = win.scale_factor() as f32; let size = win.inner_size();
        let dx = ((size.width as f32 / scale) - SETTINGS_W) / 2.0; let dy = ((size.height as f32 / scale) - SETTINGS_H) / 2.0;
        let _lmx = mx - dx; let lmy = my - dy;
        if lmy >= 20.0 && lmy <= 56.0 { let tab_w = 90.0; let start_x = (SETTINGS_W - tab_w * 4.0) / 2.0; for i in 0..4 { if _lmx >= start_x + (i as f32 * tab_w) && _lmx <= start_x + ((i+1) as f32 * tab_w) { if self.active_tab != i { self.active_tab = i; self.target_scroll_y = 0.0; changed = true; } } } }
        let content_my = lmy + if lmy >= 70.0 { self.scroll_y } else { 0.0 };
        match self.active_tab {
            0 => {
                let sy = 90.0;
                self.check_btn(_lmx, content_my, 270.0, sy + 2.0, |c| { c.global_scale = (c.global_scale - 0.05).max(0.5); }, &mut changed);
                self.check_btn(_lmx, content_my, 345.0, sy + 2.0, |c| { c.global_scale = (c.global_scale + 0.05).min(5.0); }, &mut changed);
                self.check_btn(_lmx, content_my, 270.0, sy + 52.0, |c| c.base_width -= 5.0, &mut changed);
                self.check_btn(_lmx, content_my, 345.0, sy + 52.0, |c| c.base_width += 5.0, &mut changed);
                self.check_btn(_lmx, content_my, 270.0, sy + 102.0, |c| c.base_height -= 2.0, &mut changed);
                self.check_btn(_lmx, content_my, 345.0, sy + 102.0, |c| c.base_height += 2.0, &mut changed);
                self.check_btn(_lmx, content_my, 270.0, sy + 152.0, |c| c.expanded_width -= 10.0, &mut changed);
                self.check_btn(_lmx, content_my, 345.0, sy + 152.0, |c| c.expanded_width += 10.0, &mut changed);
                self.check_btn(_lmx, content_my, 270.0, sy + 202.0, |c| c.expanded_height -= 10.0, &mut changed);
                self.check_btn(_lmx, content_my, 345.0, sy + 202.0, |c| c.expanded_height += 10.0, &mut changed);
                let sw_border_y = sy + 260.0;
                if Self::in_rect(_lmx, content_my, 326.0, sw_border_y + 3.0, 48.0, 26.0) { self.config.adaptive_border = !self.config.adaptive_border; changed = true; }
                if Self::in_rect(_lmx, content_my, 326.0, sw_border_y + 53.0, 48.0, 26.0) { self.config.motion_blur = !self.config.motion_blur; changed = true; }
                let autostart_y = sw_border_y + 100.0;
                if Self::in_rect(_lmx, content_my, 326.0, autostart_y + 3.0, 48.0, 26.0) { self.config.auto_start = !self.config.auto_start; let _ = set_autostart(self.config.auto_start); changed = true; }
                let update_y = autostart_y + 50.0;
                if Self::in_rect(_lmx, content_my, 326.0, update_y + 3.0, 48.0, 26.0) { self.config.check_for_updates = !self.config.check_for_updates; changed = true; }
                let lang_y = update_y + 50.0;
                if Self::in_rect(_lmx, content_my, 300.0, lang_y + 3.0, 75.0, 26.0) { self.config.language = if current_lang() == "zh" { "en".to_string() } else { "zh".to_string() }; set_lang(&self.config.language); changed = true; }
                if _lmx >= SETTINGS_W / 2.0 - 100.0 && _lmx <= SETTINGS_W / 2.0 + 100.0 && content_my >= lang_y + 70.0 - 20.0 && content_my <= lang_y + 70.0 + 20.0 { self.config = AppConfig::default(); changed = true; }
            }
            1 => {
                let sy = 90.0;
                if Self::in_rect(_lmx, content_my, 270.0, sy + 3.0, 105.0, 26.0) { self.config.window_effect = match self.config.window_effect { WindowEffect::None => WindowEffect::Acrylic, WindowEffect::Acrylic => WindowEffect::Mica, WindowEffect::Mica => WindowEffect::LiquidGlass, WindowEffect::LiquidGlass => WindowEffect::None }; changed = true; }
                if Self::in_rect(_lmx, content_my, 326.0, sy + 53.0, 48.0, 26.0) { self.config.smooth_transition = !self.config.smooth_transition; changed = true; }
                if Self::in_rect(_lmx, content_my, 326.0, sy + 103.0, 48.0, 26.0) { self.config.show_progress_bar = !self.config.show_progress_bar; changed = true; }
                if Self::in_rect(_lmx, content_my, 270.0, sy + 153.0, 105.0, 26.0) { self.config.progress_bar_style = match self.config.progress_bar_style { ProgressBarStyle::Gradient => ProgressBarStyle::Solid, ProgressBarStyle::Solid => ProgressBarStyle::Gradient }; changed = true; }
                if Self::in_rect(_lmx, content_my, 326.0, sy + 203.0, 48.0, 26.0) { self.config.is_solid_theme = !self.config.is_solid_theme; changed = true; }

                let preset_start_y = sy + 250.0;
                for (i, (_, c1, c2)) in THEME_PRESETS.iter().enumerate() {
                    let y = preset_start_y + (i as f32 * 45.0); if Self::in_rect(_lmx, content_my, 20.0, y - 5.0, SETTINGS_W - 40.0, 40.0) { self.config.theme_colors = vec![ThemeColor { r: c1[0], g: c1[1], b: c1[2], a: c1[3], position: 0.0 }, ThemeColor { r: c2[0], g: c2[1], b: c2[2], a: c2[3], position: 1.0 }]; changed = true; }
                }
                let custom_y = preset_start_y + (THEME_PRESETS.len() as f32 * 45.0) + 10.0;
                if Self::in_rect(_lmx, content_my, 20.0, custom_y, SETTINGS_W - 40.0, 36.0) {
                    unsafe {
                        let mut cust_colors = [windows::Win32::Foundation::COLORREF(0xFFFFFF); 16];
                        let mut cc = windows::Win32::UI::Controls::Dialogs::CHOOSECOLORW::default();
                        cc.lStructSize = std::mem::size_of::<windows::Win32::UI::Controls::Dialogs::CHOOSECOLORW>() as u32;
                        if let Some(win) = &self.window {
                            use winit::raw_window_handle::HasWindowHandle;
                            if let Ok(handle) = win.window_handle() {
                                if let winit::raw_window_handle::RawWindowHandle::Win32(h) = handle.as_raw() {
                                    cc.hwndOwner = windows::Win32::Foundation::HWND(h.hwnd.get() as *mut _);
                                }
                            }
                        }
                        let current_rgb = (self.config.theme_colors[0].r as u32) | ((self.config.theme_colors[0].g as u32) << 8) | ((self.config.theme_colors[0].b as u32) << 16);
                        cc.rgbResult = windows::Win32::Foundation::COLORREF(current_rgb);
                        cc.lpCustColors = cust_colors.as_mut_ptr();
                        cc.Flags = windows::Win32::UI::Controls::Dialogs::CC_FULLOPEN | windows::Win32::UI::Controls::Dialogs::CC_RGBINIT;

                        if windows::Win32::UI::Controls::Dialogs::ChooseColorW(&mut cc).as_bool() {
                            let r = (cc.rgbResult.0 & 0xFF) as u8;
                            let g = ((cc.rgbResult.0 >> 8) & 0xFF) as u8;
                            let b = ((cc.rgbResult.0 >> 16) & 0xFF) as u8;
                            self.config.theme_colors = vec![
                                ThemeColor { r, g, b, a: 255, position: 0.0 },
                                ThemeColor { r, g, b, a: 255, position: 1.0 }
                            ];
                            changed = true;
                        }
                    }
                }
            }
            2 => {
                let sy = 90.0; let fps_list = [0, 30, 60, 120, 144, 160, 240];
                // 减号：向左循环切换
                self.check_btn(_lmx, content_my, 270.0, sy + 2.0, |c| { 
                    let idx = fps_list.iter().position(|&f| f == c.fps_limit).unwrap_or(2); 
                    c.fps_limit = fps_list[if idx == 0 { fps_list.len() - 1 } else { idx - 1 }]; 
                }, &mut changed);
                // 加号：向右循环切换
                self.check_btn(_lmx, content_my, 345.0, sy + 2.0, |c| { 
                    let idx = fps_list.iter().position(|&f| f == c.fps_limit).unwrap_or(2); 
                    c.fps_limit = fps_list[(idx + 1) % fps_list.len()]; 
                }, &mut changed);
                if Self::in_rect(_lmx, content_my, 326.0, sy + 53.0, 48.0, 26.0) { self.config.use_gpu = !self.config.use_gpu; changed = true; }
                if Self::in_rect(_lmx, content_my, 326.0, sy + 103.0, 48.0, 26.0) { self.config.show_fps = !self.config.show_fps; changed = true; }
                if Self::in_rect(_lmx, content_my, 326.0, sy + 153.0, 48.0, 26.0) { self.config.show_gpu_status = !self.config.show_gpu_status; changed = true; }
            }
            3 => {
                let start_y = 90.0;
                if let Ok(mut pm) = crate::core::plugin::PLUGIN_MANAGER.try_lock() {
                    let mut current_y = start_y;
                    for (i, plugin) in pm.plugins.values_mut().enumerate() {
                        let y = current_y + (i as f32 * 100.0);
                        
                        // Config button
                        if plugin.has_config_ui_fn.is_some() {
                            if Self::in_rect(_lmx, content_my, SETTINGS_W - 100.0, y + 10.0, 70.0, 26.0) {
                                if let Some(open_config) = plugin.open_config_ui_fn {
                                    unsafe { open_config() };
                                }
                            }
                        }
                        
                        // Enable/Disable toggle button
                        if Self::in_rect(_lmx, content_my, SETTINGS_W - 100.0, y + 45.0, 70.0, 26.0) {
                            plugin.enabled = !plugin.enabled;
                            changed = true;
                        }
                    }
                }
            }
            4 => { if lmy >= 260.0 && lmy <= 300.0 && _lmx >= SETTINGS_W/2.0 - 100.0 && _lmx <= SETTINGS_W/2.0 + 100.0 { let _ = open::that(APP_HOMEPAGE); } }
            _ => {}
        }
        if changed { save_config(&self.config); if let Some(win) = &self.window { win.request_redraw(); } }
    }

    fn get_hover_state(&self) -> bool {
        let (mx, my) = self.logical_mouse_pos; let win = self.window.as_ref().unwrap(); let scale = win.scale_factor() as f32; let size = win.inner_size();
        let dx = ((size.width as f32 / scale) - SETTINGS_W) / 2.0; let dy = ((size.height as f32 / scale) - SETTINGS_H) / 2.0;
        let _lmx = mx - dx; let lmy = my - dy;
        lmy >= 20.0 && lmy <= 56.0 || lmy >= 70.0
    }

    fn in_rect(mx: f32, my: f32, x: f32, y: f32, w: f32, h: f32) -> bool { mx >= x && mx <= x + w && my >= y && my <= y + h }

    fn check_btn(&mut self, mx: f32, my: f32, bx: f32, by: f32, mut f: impl FnMut(&mut AppConfig), changed: &mut bool) { if mx >= bx && mx <= bx + 28.0 && my >= by && my <= by + 28.0 { f(&mut self.config); *changed = true; } }
}

impl ApplicationHandler for SettingsApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes().with_title("Settings").with_inner_size(LogicalSize::new(SETTINGS_W as f64, SETTINGS_H as f64)).with_resizable(false).with_enabled_buttons(WindowButtons::CLOSE | WindowButtons::MINIMIZE).with_window_icon(get_app_icon());
        let window = Arc::new(event_loop.create_window(attrs).unwrap()); self.window = Some(window.clone());
        let context = Context::new(window.clone()).unwrap(); let mut surface = Surface::new(&context, window.clone()).unwrap();
        let size = window.inner_size(); surface.resize(std::num::NonZeroU32::new(size.width).unwrap(), std::num::NonZeroU32::new(size.height).unwrap()).unwrap(); self.surface = Some(surface);
    }

    fn window_event(&mut self, _el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => _el.exit(),
            WindowEvent::CursorMoved { position, .. } => {
                let scale = self.window.as_ref().unwrap().scale_factor() as f32; self.logical_mouse_pos = (position.x as f32 / scale, position.y as f32 / scale);
                if let Some(win) = &self.window { win.set_cursor(if self.get_hover_state() { winit::window::CursorIcon::Pointer } else { winit::window::CursorIcon::Default }); }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let diff = match delta { winit::event::MouseScrollDelta::LineDelta(_, y) => y * 25.0, winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 };
                self.target_scroll_y -= diff; let content_h = self.get_content_height(); let _max_scroll = (content_h - (SETTINGS_H - 70.0)).max(0.0);
                self.target_scroll_y = self.target_scroll_y.clamp(0.0, _max_scroll); if let Some(win) = &self.window { win.request_redraw(); }
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => self.handle_click(),
            WindowEvent::RedrawRequested => self.draw(),
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _el: &ActiveEventLoop) {
        if let Some(win) = &self.window {
            self.frame_count += 1; let mut redraw = false;
            let tb = if self.config.adaptive_border { 1.0 } else { 0.0 }; if (tb - self.border_switch_pos).abs() > 0.01 { self.border_switch_pos += (tb - self.border_switch_pos) * 0.2; redraw = true; }
            let tbu = if self.config.motion_blur { 1.0 } else { 0.0 }; if (tbu - self.blur_switch_pos).abs() > 0.01 { self.blur_switch_pos += (tbu - self.blur_switch_pos) * 0.2; redraw = true; }
            let tas = if self.config.auto_start { 1.0 } else { 0.0 }; if (tas - self.autostart_switch_pos).abs() > 0.01 { self.autostart_switch_pos += (tas - self.autostart_switch_pos) * 0.2; redraw = true; }
            let tcu = if self.config.check_for_updates { 1.0 } else { 0.0 }; if (tcu - self.update_switch_pos).abs() > 0.01 { self.update_switch_pos += (tcu - self.update_switch_pos) * 0.2; redraw = true; }
            let tgp = if self.config.use_gpu { 1.0 } else { 0.0 }; if (tgp - self.gpu_switch_pos).abs() > 0.01 { self.gpu_switch_pos += (tgp - self.gpu_switch_pos) * 0.2; redraw = true; }
            let tsm = if self.config.smooth_transition { 1.0 } else { 0.0 }; if (tsm - self.smooth_switch_pos).abs() > 0.01 { self.smooth_switch_pos += (tsm - self.smooth_switch_pos) * 0.2; redraw = true; }
            let tpr = if self.config.show_progress_bar { 1.0 } else { 0.0 }; if (tpr - self.progress_switch_pos).abs() > 0.01 { self.progress_switch_pos += (tpr - self.progress_switch_pos) * 0.2; redraw = true; }
            let tsl = if self.config.is_solid_theme { 1.0 } else { 0.0 }; if (tsl - self.solid_theme_switch_pos).abs() > 0.01 { self.solid_theme_switch_pos += (tsl - self.solid_theme_switch_pos) * 0.2; redraw = true; }
            let tfps = if self.config.show_fps { 1.0 } else { 0.0 }; if (tfps - self.fps_switch_pos).abs() > 0.01 { self.fps_switch_pos += (tfps - self.fps_switch_pos) * 0.2; redraw = true; }
            let tgpus = if self.config.show_gpu_status { 1.0 } else { 0.0 }; if (tgpus - self.gpu_status_switch_pos).abs() > 0.01 { self.gpu_status_switch_pos += (tgpus - self.gpu_status_switch_pos) * 0.2; redraw = true; }
            
            if (self.target_scroll_y - self.scroll_y).abs() > 0.1 { self.scroll_y += (self.target_scroll_y - self.scroll_y) * 0.28; redraw = true; }
            if redraw { win.request_redraw(); }
        }
    }
}

pub fn run_settings(config: AppConfig) {
    let el = EventLoop::new().unwrap(); let mut app = SettingsApp::new(config); el.run_app(&mut app).unwrap();
}
