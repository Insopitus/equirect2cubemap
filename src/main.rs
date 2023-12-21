mod math;

use std::{fmt::Display, fs::create_dir_all, path::PathBuf};

use anyhow::Ok;
use image::{imageops, DynamicImage, EncodableLayout, ImageBuffer, Rgba, RgbaImage};
use math::Interpolation;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::math::{SphericalAngle, Vector3};

fn main() -> Result<(), anyhow::Error> {
    use clap::Parser;
    let config = Config::parse();
    let path = &config.input;
    let img = image::open(path)?;
    let width = img.width();
    let height = img.height();
    if width != height * 2 {
        panic!("Image width should be exact 2 times of image height.")
    }

    create_dir_all(&config.output)?;
    let start_time = std::time::Instant::now();
    // convert equirect to cubemaps
    let mut data = convert(&config, img);
    let elapsed = start_time.elapsed();
    println!("Convert: {:?}", elapsed);
    if config.rotate {
        let start_time = std::time::Instant::now();
        data = rotate(data);
        let elapsed = start_time.elapsed();
        println!("Rotate: {:?}", elapsed);
    }
    let start_time = std::time::Instant::now();
    let size = config.size;
    // write images to disk
    data.par_iter().for_each(|(img, side)| {
        image::save_buffer_with_format(
            config.output.join(format!("{}.{}", side, &config.format)),
            img.as_bytes(),
            size,
            size,
            image::ColorType::Rgba8,
            config.format.into(),
        )
        .unwrap();
    });
    let elapsed = start_time.elapsed();
    println!("Save: {:?}", elapsed);
    println!(
        r#"Generated images has been saved in "{}""#,
        config.output.display()
    );
    Ok(())
}

#[derive(clap::Parser, Debug)]
struct Config {
    /// the image format of the output images
    #[arg(short, long, value_enum,default_value_t = OutputFormat::Png)]
    format: OutputFormat,
    /// interpolation used when sampling source image
    #[arg(short, long,value_enum, default_value_t = Interpolation::Linear)]
    interpolation: Interpolation,
    /// the input equirectangular image's path
    input: PathBuf,
    /// the directory to put the output images in, creates if doesn't exist
    output: PathBuf,
    #[arg(short, long, default_value_t = 512)]
    /// size (px) of the output images, width = height
    size: u32,
    /// rotate to a z-up skybox if you use it in a y-up renderer
    #[arg(short, long, default_value_t = false)]
    rotate: bool,
}
#[derive(clap::ValueEnum, Clone, Debug, Copy)]
enum OutputFormat {
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
enum Side {
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
fn convert(config: &Config, img: DynamicImage) -> Vec<(ImageBuffer<Rgba<u8>, Vec<u8>>, Side)> {
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

            match side {
                Front => {
                    for x in 0..size_int {
                        let xf = x as f32;
                        for y in 0..size_int {
                            let yf = y as f32;
                            let pos = Vector3::new(0.5, xf / size - 0.5, yf / size - 0.5);
                            let spr = SphericalAngle::from_normalized_vector(pos.normalize());
                            let uv = spr.to_uv();
                            let p = interpolation.sample(&img, uv);
                            square.put_pixel(x, y, p);
                        }
                    }
                }
                Back => {
                    for x in 0..size_int {
                        let xf = x as f32;
                        for y in 0..size_int {
                            let yf = y as f32;
                            let pos = Vector3::new(-0.5, 0.5 - xf / size, yf / size - 0.5);
                            let spr = SphericalAngle::from_normalized_vector(pos.normalize());
                            let uv = spr.to_uv();
                            let p = interpolation.sample(&img, uv);
                            square.put_pixel(x, y, p);
                        }
                    }
                }
                Left => {
                    for x in 0..size_int {
                        let xf = x as f32;
                        for y in 0..size_int {
                            let yf = y as f32;
                            let pos = Vector3::new(-(xf / size - 0.5), 0.5, yf / size - 0.5);
                            // dbg!(&pos);
                            let spr = SphericalAngle::from_normalized_vector(pos.normalize());
                            // dbg!(&spr);
                            let uv = spr.to_uv();
                            // dbg!(&uv);
                            let p = interpolation.sample(&img, uv);
                            square.put_pixel(x, y, p);
                        }
                    }
                }
                Right => {
                    for x in 0..size_int {
                        let xf = x as f32;
                        for y in 0..size_int {
                            let yf = y as f32;
                            let pos = Vector3::new(xf / size - 0.5, -0.5, yf / size - 0.5);
                            // dbg!(&pos);
                            let spr = SphericalAngle::from_normalized_vector(pos.normalize());
                            // dbg!(&spr);
                            let uv = spr.to_uv();
                            // dbg!(&uv);
                            let p = interpolation.sample(&img, uv);
                            square.put_pixel(x, y, p);
                        }
                    }
                }
                Top => {
                    for x in 0..size_int {
                        let xf = x as f32;
                        for y in 0..size_int {
                            let yf = y as f32;
                            let pos = Vector3::new(xf / size - 0.5, 0.5 - yf / size, -0.5);
                            let spr = SphericalAngle::from_normalized_vector(pos.normalize());
                            let uv = spr.to_uv();
                            let p = interpolation.sample(&img, uv);
                            square.put_pixel(x, y, p);
                        }
                    }
                }
                Bottom => {
                    for x in 0..size_int {
                        let xf = x as f32;
                        for y in 0..size_int {
                            let yf = y as f32;
                            let pos = Vector3::new(xf / size - 0.5, yf / size - 0.5, 0.5);
                            let spr = SphericalAngle::from_normalized_vector(pos.normalize());
                            let uv = spr.to_uv();
                            let p = interpolation.sample(&img, uv);
                            square.put_pixel(x, y, p);
                        }
                    }
                }
            }
            (square, *side)
        })
        .collect()
}

fn rotate(
    entries: Vec<(ImageBuffer<Rgba<u8>, Vec<u8>>, Side)>,
) -> Vec<(ImageBuffer<Rgba<u8>, Vec<u8>>, Side)> {
    use imageops::*;
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
