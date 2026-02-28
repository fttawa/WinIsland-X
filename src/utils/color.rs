use windows::Win32::Graphics::Gdi::{GetDC, GetPixel, ReleaseDC, HDC};
use windows::Win32::Foundation::HWND;

pub struct ColorSampler {
    hdc: HDC,
}

impl ColorSampler {
    pub fn new() -> Self {
        let hdc = unsafe { GetDC(HWND::default()) };
        Self { hdc }
    }

    pub fn get_brightness(&self, x: i32, y: i32) -> f32 {
        let color = unsafe { GetPixel(self.hdc, x, y) };
        let r = (color.0 & 0x000000FF) as u8;
        let g = ((color.0 & 0x0000FF00) >> 8) as u8;
        let b = ((color.0 & 0x00FF0000) >> 16) as u8;
        
        let luminance = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
        (1.0 - (luminance / 255.0)).clamp(0.0, 1.0)
    }
}

impl Drop for ColorSampler {
    fn drop(&mut self) {
        unsafe {
            let _ = ReleaseDC(HWND::default(), self.hdc);
        }
    }
}

pub fn get_island_border_weights(cx: i32, cy: i32, w: f32, h: f32) -> [f32; 4] {
    let sampler = ColorSampler::new();
    
    let offset_x = (w / 2.0 + 45.0) as i32;
    let offset_y = (h / 2.0 + 45.0) as i32;

    [
        sampler.get_brightness(cx + offset_x, cy),
        sampler.get_brightness(cx, cy + offset_y),
        sampler.get_brightness(cx - offset_x, cy),
        sampler.get_brightness(cx, cy - offset_y),
    ]
}
