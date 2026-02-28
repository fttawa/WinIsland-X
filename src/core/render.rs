use skia_safe::{Color, Paint, Rect, RRect, surfaces, gradient_shader, Point};
use softbuffer::Surface;
use std::sync::Arc;
use winit::window::Window;
use crate::core::config::PADDING;

pub fn draw_island(
    surface: &mut Surface<Arc<Window>, Arc<Window>>,
    current_w: f32,
    current_h: f32,
    current_r: f32,
    os_w: u32,
    os_h: u32,
    weights: [f32; 4],
) {
    let mut buffer = surface.buffer_mut().unwrap();

    let mut sk_surface = surfaces::raster_n32_premul(skia_safe::ISize::new(os_w as i32, os_h as i32)).unwrap();

    let canvas = sk_surface.canvas();
    canvas.clear(Color::TRANSPARENT);

    let offset_x = (os_w as f32 - current_w) / 2.0;
    let offset_y = PADDING / 2.0;
    
    let rect = Rect::from_xywh(offset_x, offset_y, current_w, current_h);
    let rrect = RRect::new_rect_xy(rect, current_r, current_r);

    let mut paint = Paint::default();
    paint.set_color(Color::BLACK);
    paint.set_anti_alias(true);

    canvas.draw_rrect(rrect, &paint);

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
        
        let shader = gradient_shader::sweep(
            center,
            &colors[..],
            Some(&stops[..]),
            skia_safe::TileMode::Clamp,
            None,
            None,
            None,
        );

        if let Some(shader) = shader {
            let mut stroke_paint = Paint::default();
            stroke_paint.set_shader(shader);
            stroke_paint.set_style(skia_safe::paint::Style::Stroke);
            stroke_paint.set_stroke_width(1.3);
            stroke_paint.set_anti_alias(true);
            canvas.draw_rrect(rrect, &stroke_paint);
        }
    }

    let info = skia_safe::ImageInfo::new(
        skia_safe::ISize::new(os_w as i32, os_h as i32),
        skia_safe::ColorType::BGRA8888,
        skia_safe::AlphaType::Premul,
        None,
    );

    let dst_row_bytes = (os_w * 4) as usize;
    let u8_buffer: &mut [u8] = bytemuck::cast_slice_mut(&mut *buffer);
    let _ = sk_surface.read_pixels(&info, u8_buffer, dst_row_bytes, (0, 0));

    buffer.present().unwrap();
}
