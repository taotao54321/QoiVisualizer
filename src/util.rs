use anyhow::anyhow;
use image::RgbaImage;
use wasm_bindgen::Clamped;
use web_sys::ImageData;

pub fn create_image_data(img: &RgbaImage) -> anyhow::Result<ImageData> {
    let clamped = Clamped(&**img);

    ImageData::new_with_u8_clamped_array_and_sh(clamped, img.width(), img.height())
        .map_err(|e| anyhow!("{:?}", e))
}
