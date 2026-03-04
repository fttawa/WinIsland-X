use std::ffi::c_void;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, ReleaseDC,
    SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, GetDIBits, HBITMAP,
    SRCCOPY, CAPTUREBLT, CreatedHDC,
};
use skia_safe::{Image, ImageInfo, ColorType, AlphaType, ISize, Data, images};

pub struct ScreenCapture {
    screen_dc: CreatedHDC,
    mem_dc: CreatedHDC,
    bitmap: HBITMAP,
    width: i32,
    height: i32,
    pixels: Vec<u8>,
}

impl ScreenCapture {
    pub fn new(width: i32, height: i32) -> Option<Self> {
        unsafe {
            let screen_dc = GetDC(HWND(std::ptr::null_mut()));
            if screen_dc.is_invalid() {
                return None;
            }
            let mem_dc = CreateCompatibleDC(screen_dc);
            if mem_dc.is_invalid() {
                ReleaseDC(HWND(std::ptr::null_mut()), screen_dc);
                return None;
            }
            let bitmap = CreateCompatibleBitmap(screen_dc, width, height);
            if bitmap.is_invalid() {
                DeleteDC(mem_dc);
                ReleaseDC(HWND(std::ptr::null_mut()), screen_dc);
                return None;
            }
            SelectObject(mem_dc, bitmap);

            Some(Self {
                screen_dc,
                mem_dc,
                bitmap,
                width,
                height,
                pixels: vec![0u8; (width * height * 4) as usize],
            })
        }
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        if self.width == width && self.height == height {
            return;
        }
        unsafe {
            DeleteObject(self.bitmap);
            self.bitmap = CreateCompatibleBitmap(self.screen_dc, width, height);
            SelectObject(self.mem_dc, self.bitmap);
            self.width = width;
            self.height = height;
            self.pixels = vec![0u8; (width * height * 4) as usize];
        }
    }

    pub fn capture(&mut self, x: i32, y: i32) -> Option<Image> {
        if self.width <= 0 || self.height <= 0 { return None; }
        unsafe {
            let rop = SRCCOPY | CAPTUREBLT;
            let _ = BitBlt(self.mem_dc, 0, 0, self.width, self.height, self.screen_dc, x, y, rop);

            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: self.width,
                    biHeight: -self.height, // Top-down
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };

            let ptr = self.pixels.as_mut_ptr() as *mut c_void;
            if GetDIBits(
                self.mem_dc,
                self.bitmap,
                0,
                self.height as u32,
                Some(ptr),
                &mut bmi,
                DIB_RGB_COLORS,
            ) == 0 {
                return None;
            }

            let info = ImageInfo::new(
                ISize::new(self.width, self.height),
                ColorType::BGRA8888,
                AlphaType::Opaque,
                None,
            );
            
            // Safety: pixels is valid and size matches
            let data = Data::new_copy(&self.pixels);
            images::raster_from_data(&info, data, (self.width * 4) as usize)
        }
    }
}


impl Drop for ScreenCapture {
    fn drop(&mut self) {
        unsafe {
            DeleteObject(self.bitmap);
            DeleteDC(self.mem_dc);
            ReleaseDC(HWND(std::ptr::null_mut()), self.screen_dc);
        }
    }
}
