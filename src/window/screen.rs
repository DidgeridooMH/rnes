use crate::window::NATIVE_RESOLUTION;

#[repr(C)]
#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

const BUFFER_SIZE: usize = NATIVE_RESOLUTION.width as usize * NATIVE_RESOLUTION.height as usize;

#[derive(Copy, Clone)]
pub struct ScreenBuffer {
    pub buffer: [Pixel; BUFFER_SIZE],
}

impl Default for ScreenBuffer {
    fn default() -> Self {
        Self {
            buffer: [Pixel::default(); BUFFER_SIZE],
        }
    }
}
