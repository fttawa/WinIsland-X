use skia_safe::{Color, Paint, Rect, RRect, surfaces, gradient_shader, Point, image_filters, Surface as SkSurface, SamplingOptions, FilterMode, MipmapMode, TileMode, ISize, ClipOp};
use softbuffer::Surface;
use std::sync::Arc;
use std::cell::RefCell;
use winit::window::Window;
use crate::core::config::PADDING;
use crate::ui::expanded::main_view::{draw_main_page, get_media_palette, draw_visualizer, get_cached_media_image};
use crate::ui::expanded::tools_view::draw_tools_page;
use crate::core::smtc::MediaInfo;
thread_local! {
    static SK_SURFACE: RefCell<Option<SkSurface>> = RefCell::new(None);
}
pub fn draw_island(
    surface: &mut Surface<Arc<Window>, Arc<Window>>,
    current_w: f32,
    current_h: f32,
    current_r: f32,
    os_w: u32,
    os_h: u32,
    weights: [f32; 4],
    sigmas: (f32, f32),
    expansion_progress: f32,
    view_offset: f32,
    media: &MediaInfo,
    music_active: bool,
    global_scale: f32,
) {
    let mut buffer = surface.buffer_mut().unwrap();
    let mut sk_surface = SK_SURFACE.with(|cell| {
        let mut opt = cell.borrow_mut();
        if let Some(ref s) = *opt {
            if s.width() == os_w as i32 && s.height() == os_h as i32 { return s.clone(); }
        }
        let new_surface = surfaces::raster_n32_premul(ISize::new(os_w as i32, os_h as i32)).unwrap();
        *opt = Some(new_surface.clone());
        new_surface
    });
    let canvas = sk_surface.canvas();
    canvas.clear(Color::TRANSPARENT);
    let offset_x = (os_w as f32 - current_w) / 2.0;
    let offset_y = PADDING / 2.0;
    let rect = Rect::from_xywh(offset_x, offset_y, current_w, current_h);
    let rrect = RRect::new_rect_xy(rect, current_r, current_r);
    let has_blur = sigmas.0 > 0.1 || sigmas.1 > 0.1;
    let blur_filter = if has_blur { image_filters::blur(sigmas, None, None, None) } else { None };
    canvas.save();
    canvas.clip_rrect(rrect, ClipOp::Intersect, true);
    let mut bg_paint = Paint::default();
    bg_paint.set_color(Color::BLACK);
    bg_paint.set_anti_alias(true);
    canvas.draw_rrect(rrect, &bg_paint);

    let expanded_alpha_f = (expansion_progress.powf(2.0)).clamp(0.0, 1.0);
    let mini_alpha_f = (1.0 - expansion_progress * 1.5).clamp(0.0, 1.0);
    if expanded_alpha_f > 0.01 {
        let alpha = (expanded_alpha_f * 255.0) as u8;
        canvas.save();
        if let Some(ref filter) = blur_filter {
            let mut layer_paint = Paint::default();
            layer_paint.set_image_filter(filter.clone());
            canvas.save_layer(&skia_safe::canvas::SaveLayerRec::default().paint(&layer_paint));
        }
        canvas.save();
        canvas.translate((-view_offset * current_w, 0.0));
        draw_main_page(canvas, offset_x, offset_y, current_w, current_h, alpha, media, music_active, view_offset, global_scale);
        canvas.restore();
        canvas.save();
        canvas.translate(((1.0 - view_offset) * current_w, 0.0));
        draw_tools_page(canvas, offset_x, offset_y, current_w, current_h, alpha, view_offset, global_scale);
        canvas.restore();
        if view_offset > 0.01 && view_offset < 0.99 {
            let transition_alpha = (view_offset * (1.0 - view_offset) * 4.0 * 0.4 * alpha as f32 / 255.0 * 255.0) as u8;
            if transition_alpha > 0 {
                let mut trans_paint = Paint::default();
                trans_paint.set_color(Color::from_argb(transition_alpha, 0, 0, 0));
                canvas.draw_rrect(rrect, &trans_paint);
            }
        }
        if blur_filter.is_some() { canvas.restore(); }
        canvas.restore();
    }
    if mini_alpha_f > 0.01 && current_w > 100.0 * global_scale && music_active {
        let alpha = (mini_alpha_f * 255.0) as u8;
        if let Some(image) = get_cached_media_image(media) {
            let size = 18.0 * global_scale; 
            let ix = offset_x + 8.0 * global_scale;
            let iy = offset_y + (current_h - size) / 2.0;
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_alpha_f(alpha as f32 / 255.0);
            canvas.save();
            canvas.clip_rrect(RRect::new_rect_xy(Rect::from_xywh(ix, iy, size, size), 5.0 * global_scale, 5.0 * global_scale), ClipOp::Intersect, true);
            let sampling = SamplingOptions::new(FilterMode::Linear, MipmapMode::Linear);
            canvas.draw_image_rect_with_sampling_options(&image, None, Rect::from_xywh(ix, iy, size, size), sampling, &paint);
            canvas.restore();
        }
        let palette = get_media_palette(media);
        let viz_x = offset_x + current_w - 17.0 * global_scale;
        let viz_y = offset_y + current_h / 2.0;
        draw_visualizer(
            canvas,
            viz_x,
            viz_y,
            alpha,
            media.is_playing,
            &palette,
            &media.spectrum,
            0.55 * global_scale,
            0.08 * global_scale
        );
    }
    let total_weight: f32 = weights.iter().sum();
    if total_weight > 0.01 {
        let center = Point::new(os_w as f32 / 2.0, offset_y + current_h / 2.0);
        let colors = [
            if weights[0] > 0.85 { Color::from_argb((weights[0] * 100.0) as u8, 255, 255, 255) } else { Color::TRANSPARENT },
            if weights[1] > 0.85 { Color::from_argb((weights[1] * 100.0) as u8, 255, 255, 255) } else { Color::TRANSPARENT },
            if weights[2] > 0.85 { Color::from_argb((weights[2] * 100.0) as u8, 255, 255, 255) } else { Color::TRANSPARENT },
            if weights[3] > 0.85 { Color::from_argb((weights[3] * 100.0) as u8, 255, 255, 255) } else { Color::TRANSPARENT },
            if weights[0] > 0.85 { Color::from_argb((weights[0] * 100.0) as u8, 255, 255, 255) } else { Color::TRANSPARENT },
        ];
        let stops = [0.0, 0.25, 0.5, 0.75, 1.0];
        if let Some(shader) = gradient_shader::linear(
            (Point::new(center.x - current_w/2.0, center.y), Point::new(center.x + current_w/2.0, center.y)),
            &colors[..], Some(&stops[..]), TileMode::Clamp, None, None
        ) {
            let mut stroke_paint = Paint::default();
            stroke_paint.set_shader(shader);
            stroke_paint.set_style(skia_safe::paint::Style::Stroke);
            stroke_paint.set_stroke_width(1.3 * global_scale);
            stroke_paint.set_anti_alias(true);
            canvas.draw_rrect(rrect, &stroke_paint);
        }
    }
    canvas.restore(); 
    let info = skia_safe::ImageInfo::new(skia_safe::ISize::new(os_w as i32, os_h as i32), skia_safe::ColorType::BGRA8888, skia_safe::AlphaType::Premul, None);
    let dst_row_bytes = (os_w * 4) as usize;
    let u8_buffer: &mut [u8] = bytemuck::cast_slice_mut(&mut *buffer);
    let _ = sk_surface.read_pixels(&info, u8_buffer, dst_row_bytes, (0, 0));
    buffer.present().unwrap();
}
