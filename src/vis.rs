use image::RgbaImage;
use strum::EnumCount;

use crate::qoi::QoiChunk;
use crate::static_image::StaticImage;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct VisConfig {
    visibles: [bool; QoiChunk::COUNT],
}

impl VisConfig {
    fn new() -> Self {
        Self {
            visibles: [true; QoiChunk::COUNT],
        }
    }

    pub fn is_visible(&self, chunk: QoiChunk) -> bool {
        self.visibles[chunk as usize]
    }

    pub fn toggle_visibility(&mut self, chunk: QoiChunk) {
        let e = &mut self.visibles[chunk as usize];
        *e = !*e;
    }

    pub fn make_all_visible(&mut self) {
        self.visibles.fill(true);
    }

    pub fn make_all_invisible(&mut self) {
        self.visibles.fill(false);
    }
}

impl Default for VisConfig {
    fn default() -> Self {
        Self::new()
    }
}

pub fn visualize(img: &StaticImage, config: &VisConfig) -> RgbaImage {
    let buf_rgba: Vec<_> = img
        .chunks()
        .iter()
        .flat_map(|&chunk| {
            let [r, g, b] = if config.is_visible(chunk) {
                color_of_chunk(chunk)
            } else {
                [0, 0, 0]
            };
            [r, g, b, 0xFF]
        })
        .collect();

    RgbaImage::from_vec(img.width(), img.height(), buf_rgba)
        .expect("buffer size should be equal to `4 * width * height`")
}

pub const fn color_of_chunk(chunk: QoiChunk) -> [u8; 3] {
    const COLORS: &[[u8; 3]] = &[
        [0xFF, 0xFF, 0x00], // Index
        [0xC0, 0xC0, 0xC0], // Run8
        [0x80, 0x80, 0x80], // Run16
        [0x00, 0xFF, 0xFF], // Diff8
        [0x00, 0xC0, 0xC0], // Diff16
        [0x00, 0x80, 0x80], // Diff24
        [0xFF, 0x00, 0x00], // Color1
        [0xC0, 0x00, 0x00], // Color2
        [0x80, 0x00, 0x00], // Color3
        [0x40, 0x00, 0x00], // Color4
    ];

    COLORS[chunk as usize]
}
