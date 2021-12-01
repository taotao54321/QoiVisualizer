use gloo_file::Blob;
use image::{ImageFormat, RgbaImage};
use strum::EnumCount;

use crate::qoi::{qoi_analyze, QoiChunk};

/// `img` and `url` contain almost the same content, but don't care.
#[derive(Debug)]
pub struct StaticImage {
    name: String,
    img: RgbaImage,
    url: String,
    filesize_orig: usize,
    filesize_qoi: usize,
    chunks: Vec<QoiChunk>,
    histogram: [usize; QoiChunk::COUNT],
}

impl StaticImage {
    fn new<S1, S2>(name: S1, img: RgbaImage, url: S2, filesize_orig: usize) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let name = name.into();
        let url = url.into();

        let (filesize_qoi, chunks, histogram) = qoi_analyze(&img);

        Self {
            name,
            img,
            url,
            filesize_orig,
            filesize_qoi,
            chunks,
            histogram,
        }
    }

    pub async fn from_blob(name: impl Into<String>, blob: &Blob) -> anyhow::Result<Self> {
        let name = name.into();

        // first, check the size limitation of Data URL. (fail fast)
        let url = gloo_file::futures::read_as_data_url(blob).await?;

        let buf = gloo_file::futures::read_as_bytes(blob).await?;
        let filesize_orig = buf.len();
        let img = image::load_from_memory(&buf)?;
        let img = img.to_rgba8();

        Ok(Self::new(name, img, url, filesize_orig))
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn filesize_orig(&self) -> usize {
        self.filesize_orig
    }

    pub fn filesize_qoi(&self) -> usize {
        self.filesize_qoi
    }

    pub fn chunks(&self) -> &[QoiChunk] {
        &self.chunks
    }

    pub fn histogram(&self) -> &[usize; QoiChunk::COUNT] {
        &self.histogram
    }

    pub fn width(&self) -> u32 {
        self.img.width()
    }

    pub fn height(&self) -> u32 {
        self.img.height()
    }

    pub fn pixel_count(&self) -> usize {
        (self.width() as usize) * (self.height() as usize)
    }
}

// default image to avoid managing `Option<StaticImage>`.
impl Default for StaticImage {
    fn default() -> Self {
        const DEFAULT_PNG_NAME: &str = "default.png";
        const DEFAULT_PNG: &[u8] =
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/asset/default.png"));
        const URL_PREFIX: &str = "data:image/png;base64,";

        // make Data URL by myself to avoid async.
        let url_cap = URL_PREFIX.len() + (4 * DEFAULT_PNG.len() + 2) / 3;
        let mut url = String::with_capacity(url_cap);
        url.push_str(URL_PREFIX);
        base64::encode_config_buf(DEFAULT_PNG, base64::STANDARD, &mut url);

        let img = image::load_from_memory_with_format(DEFAULT_PNG, ImageFormat::Png)
            .expect("default png image should be valid");
        let img = img.to_rgba8();

        Self::new(DEFAULT_PNG_NAME, img, url, DEFAULT_PNG.len())
    }
}
