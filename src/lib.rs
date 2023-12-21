use image::{DynamicImage, ImageBuffer, Rgba, RgbaImage};
use rayon::prelude::*;
use std::{fmt::Display, path::PathBuf};

pub mod math;
use math::{Interpolation, SphericalAngle, Vector3};

#[derive(clap::Parser, Debug, Clone)]
pub struct Config {
    /// the image format of the output images
    #[arg(short, long, value_enum,default_value_t = OutputFormat::Png)]
    pub format: OutputFormat,
    /// interpolation used when sampling source image
    #[arg(short, long,value_enum, default_value_t = Interpolation::Linear)]
    pub interpolation: Interpolation,
    /// the input equirectangular image's path
    pub input: PathBuf,
    /// the directory to put the output images in, creates if doesn't exist
    pub output: PathBuf,
    #[arg(short, long, default_value_t = 512)]
    /// size (px) of the output images, width = height
    pub size: u32,
    /// rotate to a z-up skybox if you use it in a y-up renderer
    #[arg(short, long, default_value_t = false)]
    pub rotate: bool,
}
#[derive(clap::ValueEnum, Clone, Debug, Copy)]
pub enum OutputFormat {
    Jpg,
    Png,
    Webp,
}
impl From<OutputFormat> for image::ImageFormat {
    fn from(value: OutputFormat) -> Self {
        match value {
            OutputFormat::Jpg => image::ImageFormat::Jpeg,
            OutputFormat::Png => image::ImageFormat::Png,
            OutputFormat::Webp => image::ImageFormat::WebP,
        }
    }
}
impl Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Jpg => write!(f, "jpg"),
            OutputFormat::Png => write!(f, "png"),
            OutputFormat::Webp => write!(f, "webp"),
        }
    }
}

#[derive(Clone, Copy)]
pub enum Side {
    Front,
    Back,
    Left,
    Right,
    Top,
    Bottom,
}
impl Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Side::Front => write!(f, "front"),
            Side::Back => write!(f, "back"),
            Side::Left => write!(f, "left"),
            Side::Right => write!(f, "right"),
            Side::Top => write!(f, "top"),
            Side::Bottom => write!(f, "bottom"),
        }
    }
}

/// convert 1 equirect image to cubemaps (6 squared images)
pub fn convert(config: &Config, img: DynamicImage) -> Vec<(ImageBuffer<Rgba<u8>, Vec<u8>>, Side)> {
    // use rayon::ParIter;
    use Side::*;
    let size = config.size;
    let interpolation = &config.interpolation;
    [Front, Back, Left, Right, Top, Bottom]
        .par_iter()
        .map(|side| {
            let size_int = size;
            let size = size as f32;
            let mut square = RgbaImage::new(size_int, size_int);
            for x in 0..size_int {
                let xf = x as f32;
                for y in 0..size_int {
                    let yf = y as f32;
                    let pos = match side {
                        Front => Vector3::new(0.5, xf / size - 0.5, yf / size - 0.5),
                        Back => Vector3::new(-0.5, 0.5 - xf / size, yf / size - 0.5),
                        Left => Vector3::new(-(xf / size - 0.5), 0.5, yf / size - 0.5),
                        Right => Vector3::new(xf / size - 0.5, -0.5, yf / size - 0.5),
                        Top => Vector3::new(xf / size - 0.5, 0.5 - yf / size, -0.5),
                        Bottom => Vector3::new(xf / size - 0.5, yf / size - 0.5, 0.5),
                    };
                    let spr = SphericalAngle::from_normalized_vector(pos.normalize());
                    let uv = spr.to_uv();
                    let p = interpolation.sample(&img, uv);
                    square.put_pixel(x, y, p);
                }
            }
            (square, *side)
        })
        .collect()
}

pub fn rotate(
    entries: Vec<(ImageBuffer<Rgba<u8>, Vec<u8>>, Side)>,
) -> Vec<(ImageBuffer<Rgba<u8>, Vec<u8>>, Side)> {
    use image::imageops::*;
    entries
        .into_par_iter()
        .map(|(img, side)| {
            let image = match side {
                Side::Top => img,
                Side::Bottom => rotate180(&img),
                Side::Left => rotate180(&img),
                Side::Right => img,
                Side::Front => rotate270(&img),
                Side::Back => rotate90(&img),
            };
            (image, side)
        })
        .collect()
}
