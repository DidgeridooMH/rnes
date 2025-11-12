#![allow(dead_code)]

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WindowUniform {
    region_aspect: f32,
    window_aspect: f32,
}

impl WindowUniform {
    pub fn new(region_aspect: f32, window_aspect: f32) -> Self {
        Self {
            region_aspect,
            window_aspect,
        }
    }
}
