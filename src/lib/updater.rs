use std::io::{stdin, Write};
use std::fs::File;
use std::string;
use image::{self, DynamicImage, GenericImageView};
use zstd::stream::Encoder;
use crate::lib::decoder_old;

fn encode_with_zstd(input_data: &[u8]) -> Vec<u8> {
    println!("Sıkıştırma öncesi boyut: {}", input_data.len());
    let mut compressed_data = Vec::new();
    let mut encoder = Encoder::new(&mut compressed_data, 11).expect("Sıkıştırıcı başlatılamadı");
    encoder.write_all(input_data).expect("Sıkıştırma hatası");
    encoder.finish().expect("Sıkıştırıcı bitirme hatası");
    println!("Sıkıştırma sonrası boyut: {}", compressed_data.len());
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

    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y).0;
            img_data.extend_from_slice(&pixel);
        }
    }

    let info = format!("\nWidth:{}\nHeight:{}\n", width, height);
    img_data.extend_from_slice(info.as_bytes());

    let compressed_data = encode_with_zstd(&img_data);

    let mut file = File::create(format!("{}.vfx", file_name.trim())).unwrap();
    file.write_all(&compressed_data).expect("Yazma hatası");
}
