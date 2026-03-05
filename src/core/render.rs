use skia_safe::{Color, Paint, Rect, RRect, image_filters, Surface as SkSurface, SamplingOptions, FilterMode, MipmapMode, ClipOp, gradient_shader, Point, TileMode, RuntimeEffect, Data, FontStyle};
use crate::core::config::{PADDING, TOP_OFFSET, AppConfig, WindowEffect};
use crate::ui::expanded::main_view::{draw_main_page, get_media_palette, draw_visualizer, get_cached_media_image, draw_text_cached};
use crate::ui::expanded::tools_view::draw_tools_page;
use crate::core::smtc::MediaInfo;
use std::cell::RefCell;

thread_local! {
    static LIQUID_EFFECT: RefCell<Option<RuntimeEffect>> = RefCell::new(RuntimeEffect::make_for_shader(
        "
        uniform float u_data[11];
        half4 main(float2 fragCoord) {
            float2 u_res = float2(u_data[0], u_data[1]);
            float u_time = u_data[2];
            half4 u_c1 = half4(u_data[3], u_data[4], u_data[5], u_data[6]);
            half4 u_c2 = half4(u_data[7], u_data[8], u_data[9], u_data[10]);
            
            float2 uv = fragCoord / u_res;
            float2 p = uv * 2.0 - 1.0;
            p.x *= u_res.x / u_res.y;
            
            float speed = u_time * 0.5;
            for(int i=1; i<5; i++) {
                p.x += 0.3 / float(i) * sin(float(i) * 3.0 * p.y + speed);
                p.y += 0.3 / float(i) * cos(float(i) * 3.0 * p.x + speed);
            }
            
            float val = 0.5 + 0.5 * sin(p.x + p.y);
            half4 color = mix(u_c1 * 0.4, u_c2 * 0.6, val);
            
            // Physical Gloss
            float light = pow(val, 8.0) * 0.5;
            color += half4(light, light, light, 0.0);
            
            // Transparency control
            color.a = 0.45; 
            color.rgb *= color.a; // Premultiply
            return color;
        }
        ", None
    ).ok());

    static ACRYLIC_EFFECT: RefCell<Option<RuntimeEffect>> = RefCell::new(RuntimeEffect::make_for_shader(
        "
        uniform float u_data[4]; // w, h, time, opacity
        half4 main(float2 fragCoord) {
            float2 uv = fragCoord;
            // Fast noise for acrylic grain
            float n = fract(sin(dot(uv, float2(12.9898, 78.233))) * 43758.5453);
            float opacity = u_data[3];
            float3 base_color = float3(0.08, 0.08, 0.1); // Deep dark blue-ish
            
            // Add subtle noise to simulate acrylic texture
            float3 color = base_color + (n - 0.5) * 0.04;
            
            // Soft gradient
            float2 p = fragCoord / float2(u_data[0], u_data[1]);
            color += (1.0 - p.y) * 0.05;
            
            return half4(color * opacity, opacity);
        }
        ", None
    ).ok());
}

pub fn draw_island(
    sk_surface: &mut SkSurface,
    current_w: f32,
    current_h: f32,
    current_r: f32,
    os_w: u32,
    _os_h: u32,
    _weights: [f32; 4],
    sigmas: (f32, f32),
    expansion_progress: f32,
    view_offset: f32,
    media: &MediaInfo,
    music_active: bool,
    global_scale: f32,
    tool_hovers: &[f32; 15],
    tool_presses: &[f32; 15],
    current_lyric: &str,
    old_lyric: &str,
    lyric_transition: f32,
    _use_blur: bool,
    hide_progress: f32,
    config: &AppConfig,
    time: f32,
    fps: f32,
) {
    let canvas = sk_surface.canvas();
    canvas.clear(Color::TRANSPARENT);
    
    let offset_x = (os_w as f32 - current_w) / 2.0;
    let base_y = PADDING / 2.0;
    let hidden_peek_h = (5.0 * global_scale).max(3.0);
    let hide_distance = (current_h - hidden_peek_h + TOP_OFFSET as f32).max(0.0);
    let offset_y = base_y - (hide_progress * hide_distance);

    let rect = Rect::from_xywh(offset_x, offset_y, current_w, current_h);
    let rrect = RRect::new_rect_xy(rect, current_r, current_r);

    // --- 1. Draw Outer Glow (泛光) ---
    if config.window_effect != WindowEffect::None {
        let mut glow_paint = Paint::default();
        glow_paint.set_anti_alias(true);
        let glow_color = if config.theme_colors.len() >= 1 {
            let c = &config.theme_colors[0];
            Color::from_argb(60, c.r, c.g, c.b)
        } else {
            Color::from_argb(40, 255, 255, 255)
        };
        glow_paint.set_image_filter(image_filters::blur((8.0 * global_scale, 8.0 * global_scale), None, None, None));
        glow_paint.set_color(glow_color);
        // 创建一个外扩的矩形用于渲染泛光
        let mut glow_rect = rect;
        glow_rect.outset((2.0 * global_scale, 2.0 * global_scale));
        canvas.draw_rrect(RRect::new_rect_xy(glow_rect, current_r + 2.0, current_r + 2.0), &glow_paint);
    }
    
    canvas.save();
    canvas.clip_rrect(rrect, ClipOp::Intersect, true);

    let mut bg_paint = Paint::default();
    bg_paint.set_anti_alias(true);

    let is_glass = config.window_effect != WindowEffect::None;
    let mut shader_applied = false;

    if is_glass {
        if config.window_effect == WindowEffect::LiquidGlass && config.theme_colors.len() >= 2 {
            LIQUID_EFFECT.with(|cell| {
                if let Some(effect) = cell.borrow().as_ref() {
                    let c1 = &config.theme_colors[0];
                    let c2 = &config.theme_colors[1];
                    let uniforms: [f32; 11] = [
                        current_w, current_h, time,
                        c1.r as f32 / 255.0, c1.g as f32 / 255.0, c1.b as f32 / 255.0, 1.0,
                        c2.r as f32 / 255.0, c2.g as f32 / 255.0, c2.b as f32 / 255.0, 1.0,
                    ];
                    let data = Data::new_copy(bytemuck::cast_slice(&uniforms));
                    if let Some(shader) = effect.make_shader(&data, &[], None) {
                        let local_matrix = skia_safe::Matrix::translate((offset_x, offset_y));
                        bg_paint.set_shader(shader.with_local_matrix(&local_matrix));
                        shader_applied = true;
                    }
                }
            });
        } else if config.window_effect == WindowEffect::Acrylic || config.window_effect == WindowEffect::Mica {
            ACRYLIC_EFFECT.with(|cell| {
                if let Some(effect) = cell.borrow().as_ref() {
                    let opacity = if config.window_effect == WindowEffect::Acrylic { 0.65 } else { 0.85 };
                    let uniforms: [f32; 4] = [current_w, current_h, time, opacity];
                    let data = Data::new_copy(bytemuck::cast_slice(&uniforms));
                    if let Some(shader) = effect.make_shader(&data, &[], None) {
                        let local_matrix = skia_safe::Matrix::translate((offset_x, offset_y));
                        bg_paint.set_shader(shader.with_local_matrix(&local_matrix));
                        shader_applied = true;
                    }
                }
            });
        }
    }

    if !shader_applied {
        if config.is_solid_theme && config.theme_colors.len() >= 1 {
            let c = &config.theme_colors[0];
            bg_paint.set_color(Color::from_argb(c.a, c.r, c.g, c.b));
        } else if is_glass {
            bg_paint.set_color(Color::from_argb(180, 20, 20, 25)); // Dark translucent
        } else if config.theme_colors.len() >= 2 {
            let colors: Vec<Color> = config.theme_colors.iter().map(|c| Color::from_argb(c.a, c.r, c.g, c.b)).collect();
            let pos: Vec<f32> = config.theme_colors.iter().map(|c| c.position).collect();
            let shader = gradient_shader::linear(
                (Point::new(offset_x, offset_y), Point::new(offset_x + current_w, offset_y + current_h)),
                colors.as_slice(), Some(pos.as_slice()), TileMode::Clamp, None, None
            );
            bg_paint.set_shader(shader);
        } else {
            bg_paint.set_color(Color::BLACK);
        }
    }

    canvas.draw_rrect(rrect, &bg_paint);

    // --- 2. Advanced Glass Layers ---
    if is_glass {
        // Luminosity Layer (亮层，模拟光感)
        let mut lum_paint = Paint::default();
        lum_paint.set_anti_alias(true);
        lum_paint.set_blend_mode(skia_safe::BlendMode::Screen);
        let lum_shader = gradient_shader::radial(
            Point::new(offset_x + current_w * 0.5, offset_y),
            current_w * 0.8,
            [Color::from_argb(30, 255, 255, 255), Color::TRANSPARENT].as_slice(),
            None, TileMode::Clamp, None, None
        );
        lum_paint.set_shader(lum_shader);
        canvas.draw_rrect(rrect, &lum_paint);

        // Top Rim Highlight (顶部物理高光)
        let mut rim_paint = Paint::default();
        rim_paint.set_anti_alias(true);
        rim_paint.set_style(skia_safe::PaintStyle::Stroke);
        rim_paint.set_stroke_width(0.8 * global_scale);
        let rim_shader = gradient_shader::linear(
            (Point::new(offset_x, offset_y), Point::new(offset_x, offset_y + 10.0)),
            [Color::from_argb(120, 255, 255, 255), Color::TRANSPARENT].as_slice(), None, TileMode::Clamp, None, None
        );
        rim_paint.set_shader(rim_shader);
        canvas.draw_rrect(rrect, &rim_paint);
        
        // Border
        let mut border_paint = Paint::default();
        border_paint.set_anti_alias(true);
        border_paint.set_style(skia_safe::PaintStyle::Stroke);
        border_paint.set_stroke_width(1.2 * global_scale);
        border_paint.set_color(Color::from_argb(60, 255, 255, 255));
        canvas.draw_rrect(rrect, &border_paint);
    }

    let expanded_alpha_f = (expansion_progress.powf(2.0)).clamp(0.0, 1.0) * (1.0 - hide_progress);
    let mini_alpha_f = (1.0 - expansion_progress * 1.5).clamp(0.0, 1.0) * (1.0 - hide_progress);
    let viz_h_scale = 0.45 + (1.0 - 0.45) * expansion_progress;

    if expanded_alpha_f > 0.01 {
        let alpha = (expanded_alpha_f * 255.0) as u8;
        canvas.save();
        if sigmas.0 > 0.1 || sigmas.1 > 0.1 {
            let mut layer_paint = Paint::default();
            layer_paint.set_image_filter(image_filters::blur(sigmas, None, None, None));
            canvas.save_layer(&skia_safe::canvas::SaveLayerRec::default().paint(&layer_paint));
        }
        canvas.translate((-view_offset * current_w, 0.0));
        draw_main_page(canvas, offset_x, offset_y, current_w, current_h, alpha, media, music_active, view_offset, global_scale, expansion_progress, viz_h_scale * global_scale, fps, config);
        if sigmas.0 > 0.1 || sigmas.1 > 0.1 { canvas.restore(); }
        canvas.restore();
        
        canvas.save();
        canvas.translate(((1.0 - view_offset) * current_w, 0.0));
        draw_tools_page(canvas, offset_x, offset_y, current_w, current_h, alpha, view_offset, global_scale, tool_hovers, tool_presses);
        canvas.restore();
    }

    if mini_alpha_f > 0.01 && current_w > 100.0 * global_scale && music_active {
        let alpha = (mini_alpha_f * 255.0) as u8;
        if let Some(image) = get_cached_media_image(media) {
            let size = 18.0 * global_scale; 
            let ix = offset_x + 8.0 * global_scale; let iy = offset_y + (current_h - size) / 2.0;
            let mut paint = Paint::default(); paint.set_anti_alias(true); paint.set_alpha_f(alpha as f32 / 255.0);
            canvas.save();
            canvas.clip_rrect(RRect::new_rect_xy(Rect::from_xywh(ix, iy, size, size), 5.0 * global_scale, 5.0 * global_scale), ClipOp::Intersect, true);
            canvas.draw_image_rect_with_sampling_options(&image, None, Rect::from_xywh(ix, iy, size, size), SamplingOptions::new(FilterMode::Linear, MipmapMode::Linear), &paint);
            canvas.restore();
        }
        let palette = get_media_palette(media);
        let viz_x = offset_x + current_w - 17.0 * global_scale; let viz_y = offset_y + current_h / 2.0;
        draw_visualizer(canvas, viz_x, viz_y, alpha, media.is_playing, &palette, &media.spectrum, 0.55 * global_scale, viz_h_scale * global_scale, (0.6, 0.08));

        if !current_lyric.is_empty() || !old_lyric.is_empty() {
            let lyric_fade_f = (1.0 - expansion_progress * 2.5).clamp(0.0, 1.0);
            let alpha = (alpha as f32 * lyric_fade_f) as u8;
            if alpha > 0 {
                let space_left = offset_x + 30.0 * global_scale; let space_right = offset_x + current_w - 29.0 * global_scale;
                let available_w = space_right - space_left; let text_x = space_left + available_w / 2.0;
                canvas.save();
                canvas.clip_rect(Rect::from_xywh(space_left, offset_y, available_w, current_h), ClipOp::Intersect, true);
                let text_y = offset_y + current_h / 2.0 + 4.0 * global_scale;
                if lyric_transition < 1.0 && !old_lyric.is_empty() {
                    let mut text_paint = Paint::default(); text_paint.set_anti_alias(true);
                    let fade_alpha = (alpha as f32 * (1.0 - lyric_transition)) as u8;
                    text_paint.set_color(Color::from_argb(fade_alpha, 255, 255, 255));
                    let blur_sigma = lyric_transition * 10.0 * global_scale;
                    if blur_sigma > 0.1 { text_paint.set_image_filter(image_filters::blur((blur_sigma, 0.0), None, None, None)); }
                    draw_text_cached(canvas, old_lyric, (text_x, text_y - (10.0 * global_scale * lyric_transition)), 12.0 * global_scale, FontStyle::normal(), &text_paint, true, available_w);
                }
                if !current_lyric.is_empty() {
                    let mut text_paint = Paint::default(); text_paint.set_anti_alias(true);
                    let fade_alpha = (alpha as f32 * lyric_transition) as u8;
                    text_paint.set_color(Color::from_argb(fade_alpha, 255, 255, 255));
                    let blur_sigma = (1.0 - lyric_transition) * 10.0 * global_scale;
                    if blur_sigma > 0.1 { text_paint.set_image_filter(image_filters::blur((blur_sigma, 0.0), None, None, None)); }
                    draw_text_cached(canvas, current_lyric, (text_x, text_y + (10.0 * global_scale * (1.0 - lyric_transition))), 12.0 * global_scale, FontStyle::normal(), &text_paint, true, available_w);
                }
                canvas.restore();
            }
        }
    }
    
    // FINAL RESTORE for the clip_rrect save
    canvas.restore();
}
