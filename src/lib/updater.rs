use std::io::Write;
use std::fs::File;
use std::string;
use image::{self, DynamicImage, GenericImageView};
use zstd::stream::Encoder;
use crate::lib::decoder_old;

fn encode_with_zstd(input_data: &[u8]) -> Vec<u8> {
    let mut compressed_data = Vec::new();
    let mut encoder = Encoder::new(&mut compressed_data, 11).expect("Failed to initialize encoder");
    encoder.write_all(input_data).expect("Compression error");
    encoder.finish().expect("Error finishing encoder");
    compressed_data
}

pub fn update(file_path: &string::String) {
    let image = decoder_old::decode(file_path);

    let file_path = file_path.clone().replace(".vfx", "");
    encode(image, &file_path);
}

fn encode(img: DynamicImage, file_name: &str) {
    let mut img_data = Vec::new();
    let (width, height) = img.dimensions();

    let signature = "0x56-0x46-0x58: 0x03";

    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y).0;
            img_data.extend_from_slice(&pixel);
        }
    }

    let height_hex = hex::encode("Height");
    let width_hex = hex::encode("Width");

    let info = format!("\n{}: {}\n{}: {}\n{}", height_hex, height, width_hex, width, signature);
    img_data.extend_from_slice(info.as_bytes());

    let compressed_data = encode_with_zstd(&img_data);

    let mut file = match File::create(format!("{}.vfx", file_name.trim())) {
        Ok(f) => f,
        Err(e) => panic!("Error while creating file: {}", e),
    };

    if let Err(e) = file.write_all(&compressed_data) {
        panic!("Write error: {}", e);
    }
}
