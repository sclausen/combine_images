use std::{env, path::PathBuf};

use image::{
    imageops::FilterType::Triangle, DynamicImage, GenericImageView, ImageError, Rgba, RgbaImage,
};

use rand::{distributions::Alphanumeric, Rng};

#[derive(Debug)]
pub enum ImageDataErrors {
    DifferentImageFormats,
    BufferToSmall,
    UnableToReadImageFromPath(std::io::Error),
    UnableToDecodeImage(ImageError),
    UnableToFormatImage(String),
    UnableToSaveImage(ImageError),
    DownloadError(reqwest::Error),
}

struct FloatingImage {
    width: u32,
    height: u32,
    data: Vec<u8>,
    name: String,
}

impl FloatingImage {
    fn new(width: u32, height: u32, name: String) -> Self {
        let buffer_capacity = height * width * 4;
        let buffer = Vec::with_capacity(buffer_capacity.try_into().unwrap());
        FloatingImage {
            width,
            height,
            data: buffer,
            name,
        }
    }

    fn set_data(&mut self, data: Vec<u8>) -> Result<(), ImageDataErrors> {
        if data.len() > self.data.capacity() {
            return Err(ImageDataErrors::BufferToSmall);
        }
        self.data = data;
        Ok(())
    }
}

pub async fn process_image(url1: String, url2: String) -> Result<String, ImageDataErrors> {
    let base = download_and_decode_image(url1).await?;
    let overlay = download_and_decode_image(url2).await?;

    let (base, overlay) = common_denominator(base, overlay);
    let output_name = generate_random_filename();
    let mut output = FloatingImage::new(base.width(), base.height(), output_name.clone());
    let combined_data = combine_images(base, overlay);
    output.set_data(combined_data)?;

    if let Err(e) = image::save_buffer_with_format(
        format!("images/{}", output.name),
        &output.data,
        output.width,
        output.height,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    ) {
        return Err(ImageDataErrors::UnableToSaveImage(e));
    }

    Ok(output_name)
}

fn min(dim_1: (u32, u32), dim_2: (u32, u32)) -> (u32, u32) {
    let pix_1 = dim_1.0 * dim_1.1;
    let pix_2 = dim_2.0 * dim_2.1;
    return if pix_1 < pix_2 { dim_1 } else { dim_2 };
}

fn common_denominator(base: DynamicImage, overlay: DynamicImage) -> (DynamicImage, DynamicImage) {
    let (width, height) = min(base.dimensions(), overlay.dimensions());

    if overlay.dimensions() == (width, height) {
        (base.resize_exact(width, height, Triangle), overlay)
    } else {
        (overlay.resize_exact(width, height, Triangle), base)
    }
}

fn combine_images(background: DynamicImage, overlay: DynamicImage) -> Vec<u8> {
    let mut background = background.to_rgba8();
    let overlay = overlay.to_rgba8();

    overlay_image(&mut background, &overlay);

    background.into_vec()
}

fn overlay_image(background: &mut RgbaImage, overlay: &RgbaImage) {
    for (x, y, pixel) in overlay.enumerate_pixels() {
        if x < background.width() && y < background.height() {
            let base_pixel = background.get_pixel_mut(x, y);
            apply_overlay(base_pixel, pixel);
        }
    }
}

fn apply_overlay(base_pixel: &mut Rgba<u8>, overlay_pixel: &Rgba<u8>) {
    let alpha = overlay_pixel[3] as f32 / 255.0;
    let inv_alpha = 1.0 - alpha;

    for i in 0..3 {
        base_pixel[i] = (overlay_pixel[i] as f32 * alpha + base_pixel[i] as f32 * inv_alpha) as u8;
    }
}

async fn download_and_decode_image(url: String) -> Result<DynamicImage, ImageDataErrors> {
    let response = reqwest::get(&url)
        .await
        .map_err(ImageDataErrors::DownloadError)?;
    let bytes = response
        .bytes()
        .await
        .map_err(ImageDataErrors::DownloadError)?;
    let image = image::load_from_memory(&bytes).map_err(ImageDataErrors::UnableToDecodeImage)?;
    Ok(image)
}

fn generate_random_filename() -> String {
    let rand_string: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();
    format!("{}.png", rand_string)
}

pub fn create_path_from_current_dir(subdir: &str) -> Result<String, std::io::Error> {
    let current_dir = env::current_dir()?;

    let mut path = PathBuf::from(current_dir);

    path.push(subdir);

    let path_str = path
        .to_str()
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to convert path to string.",
            )
        })?
        .to_owned();

    Ok(path_str)
}
