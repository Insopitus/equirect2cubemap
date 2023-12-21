use std::fs::create_dir_all;

use anyhow::Ok;
use equirect2cubemap::{convert, rotate, Config};

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

    use image::EncodableLayout as _;
    use rayon::prelude::*;

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
