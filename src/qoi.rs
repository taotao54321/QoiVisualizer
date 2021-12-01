use image::{GenericImageView, Rgba};
use strum::EnumCount;
use strum_macros::{EnumCount as EnumCountMacros, EnumIter};

use crate::pixel::{DiffOrColor, PixelDict, PixelDiff, QoiPixel};

const QOI_HEADER_LEN: usize = 14;
const QOI_PADDING_LEN: usize = 4;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, EnumCountMacros, EnumIter)]
pub enum QoiChunk {
    Index = 0,
    Run8,
    Run16,
    Diff8,
    Diff16,
    Diff24,
    Color1, // QOI_COLOR with 1 component. (same as below)
    Color2,
    Color3,
    Color4,
}

impl QoiChunk {
    pub fn name(self) -> &'static str {
        match self {
            Self::Index => "QOI_INDEX",
            Self::Run8 => "QOI_RUN_8",
            Self::Run16 => "QOI_RUN_16",
            Self::Diff8 => "QOI_DIFF_8",
            Self::Diff16 => "QOI_DIFF_16",
            Self::Diff24 => "QOI_DIFF_24",
            Self::Color1 => "QOI_COLOR (2-Bytes)",
            Self::Color2 => "QOI_COLOR (3-Bytes)",
            Self::Color3 => "QOI_COLOR (4-Bytes)",
            Self::Color4 => "QOI_COLOR (5-Bytes)",
        }
    }
}

/// returns (filesize_qoi, chunks, histogram).
pub fn qoi_analyze<I>(img: &I) -> (usize, Vec<QoiChunk>, [usize; QoiChunk::COUNT])
where
    I: GenericImageView<Pixel = Rgba<u8>>,
{
    let pixel_count = (img.width() as usize)
        .checked_mul(img.height() as usize)
        .expect("pixel count should not exceed usize::MAX");

    let pixels = img.pixels().map(|(_, _, Rgba(rgba))| QoiPixel::from(rgba));

    let mut chunks = Vec::<QoiChunk>::with_capacity(pixel_count);

    let mut enc = Analyzer::new(&mut chunks);
    for px in pixels {
        enc.update(px);
    }
    let filesize = enc.finalize();

    let mut histogram = [0; QoiChunk::COUNT];
    for &chunk in &chunks {
        histogram[chunk as usize] += 1;
    }

    (filesize, chunks, histogram)
}

const RUN_MAX: u16 = 33 + 0x1FFF;

#[derive(Debug)]
struct Analyzer<'a> {
    filesize: usize,
    chunks: &'a mut Vec<QoiChunk>,
    px_prev: QoiPixel,
    dict: PixelDict,
    run: u16,
}

impl<'a> Analyzer<'a> {
    fn new(chunks: &'a mut Vec<QoiChunk>) -> Self {
        Analyzer {
            filesize: QOI_HEADER_LEN + QOI_PADDING_LEN,
            chunks,
            px_prev: QoiPixel::new(0, 0, 0, 255),
            dict: PixelDict::new(),
            run: 0,
        }
    }

    fn update(&mut self, px: QoiPixel) {
        if px == self.px_prev {
            self.run += 1;
            if self.run == RUN_MAX {
                self.flush_run();
            }
            return;
        }

        self.flush_run();

        let hash = PixelDict::hash(px);

        if px == self.dict[hash] {
            self.filesize += 1;
            self.chunks.push(QoiChunk::Index);
        } else {
            let chunk = match px.sub(self.px_prev) {
                DiffOrColor::Diff(PixelDiff::Diff8(_)) => {
                    self.filesize += 1;
                    QoiChunk::Diff8
                }
                DiffOrColor::Diff(PixelDiff::Diff16(_)) => {
                    self.filesize += 2;
                    QoiChunk::Diff16
                }
                DiffOrColor::Diff(PixelDiff::Diff24 { .. }) => {
                    self.filesize += 3;
                    QoiChunk::Diff24
                }
                DiffOrColor::Color(mask) => {
                    let n = mask.count_ones();
                    self.filesize += (n as usize) + 1;
                    match n {
                        1 => QoiChunk::Color1,
                        2 => QoiChunk::Color2,
                        3 => QoiChunk::Color3,
                        4 => QoiChunk::Color4,
                        _ => unreachable!(),
                    }
                }
            };
            self.chunks.push(chunk);

            self.dict[hash] = px;
        }

        self.px_prev = px;
    }

    fn finalize(mut self) -> usize {
        self.flush_run();

        self.filesize
    }

    fn flush_run(&mut self) {
        match self.run {
            0 => {}
            1..=32 => {
                self.filesize += 1;
                self.chunks
                    .extend(std::iter::repeat(QoiChunk::Run8).take(usize::from(self.run)))
            }
            33..=RUN_MAX => {
                self.filesize += 2;
                self.chunks
                    .extend(std::iter::repeat(QoiChunk::Run16).take(usize::from(self.run)))
            }
            _ => unreachable!(),
        }

        self.run = 0;
    }
}
