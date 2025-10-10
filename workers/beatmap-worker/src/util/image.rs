use std::collections::HashMap;
use std::io::Cursor;
use image::codecs::png::PngEncoder;
use image::{ImageEncoder, ImageFormat, ImageReader, PixelWithColorType, Rgb, RgbImage};

use crate::parsing::BackgroundData;

const SUPPORTED_FORMATS: [ImageFormat; 3] = [ImageFormat::Png, ImageFormat::Jpeg, ImageFormat::Bmp];

pub fn make_thumbnail_png(
    raw_image: &[u8],
    bg_data: &Option<BackgroundData>,
) -> Result<Vec<u8>, String> {
    let reader = ImageReader::new(Cursor::new(raw_image))
        .with_guessed_format()
        .map_err(|e| e.to_string())?;

    if let Some(fmt) = reader.format() {
        if !SUPPORTED_FORMATS.contains(&fmt) {
            return Err(format!(
                "Unsupported background image format: {:?}. Use png, bmp, or jpeg.",
                fmt
            ));
        }
    }

    let img = reader.decode().map_err(|e| e.to_string())?;
    let size = (img.width(), img.height());
    let rgb = img.to_rgb8();
    let processed = replace_image_channels(rgb, size, bg_data);

    let mut out = Vec::new();
    PngEncoder::new(&mut out)
        .write_image(
            processed.as_ref(),
            size.0,
            size.1,
            <Rgb<u8> as PixelWithColorType>::COLOR_TYPE,
        )
        .map_err(|e| e.to_string())?;
    Ok(out)
}

fn replace_image_channels(
    mut img_buffer: RgbImage,
    size: (u32, u32),
    bg_data: &Option<BackgroundData>,
) -> RgbImage {
    let Some(bg) = bg_data else {
        return img_buffer;
    };
    let mut channels: HashMap<[u8; 3], [u8; 3]> = HashMap::new();
    if let Some(channel) = &bg.red_channel { channels.insert([255, 0, 0], *channel); }
    if let Some(channel) = &bg.green_channel { channels.insert([0, 255, 0], *channel); }
    if let Some(channel) = &bg.blue_channel { channels.insert([0, 0, 255], *channel); }
    if let Some(channel) = &bg.magenta_channel { channels.insert([255, 0, 255], *channel); }
    if let Some(channel) = &bg.cyan_channel { channels.insert([0, 255, 255], *channel); }
    if let Some(channel) = &bg.yellow_channel { channels.insert([255, 255, 0], *channel); }
    channels.insert([0, 0, 0], [0, 0, 0]);

    for (i, pixel) in img_buffer.pixels_mut().enumerate() {
        if let Some(replacement) = channels.get(&pixel.0) {
            pixel.0 = *replacement;
        } else {
            let i = i as u32;
            pixel.0 = if (i % size.0) % 2 == 0 && (i / size.0) % 2 == 0 {
                [0, 0, 0]
            } else {
                [255, 0, 255]
            }
        }
    }
    img_buffer
}
