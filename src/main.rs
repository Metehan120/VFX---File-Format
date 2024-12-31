use std::io::{stdin, Write, Read};
use std::fs::File;
use image::{self, DynamicImage, GenericImageView, RgbaImage, Rgba};
use minifb::{Window, WindowOptions};
use zstd::stream::{Encoder, Decoder};

fn encode_with_zstd(input_data: &[u8]) -> Vec<u8> {
    println!("Sıkıştırma öncesi boyut: {}", input_data.len());
    let mut compressed_data = Vec::new();
    let mut encoder = Encoder::new(&mut compressed_data, 8).expect("Sıkıştırıcı başlatılamadı");
    encoder.write_all(input_data).expect("Sıkıştırma hatası");
    encoder.finish().expect("Sıkıştırıcı bitirme hatası");
    println!("Sıkıştırma sonrası boyut: {}", compressed_data.len());
    compressed_data
}

fn decode_with_zstd(compressed_data: &[u8]) -> Vec<u8> {
    println!("Çözme işlemi başlatıldı. Sıkıştırılmış veri boyutu: {}", compressed_data.len());
    let mut decompressed_data = Vec::new();
    let mut decoder = Decoder::new(compressed_data).expect("Çözücü başlatılamadı");
    decoder.read_to_end(&mut decompressed_data).expect("Çözme hatası");
    println!("Çözme sonrası veri boyutu: {}", decompressed_data.len());
    decompressed_data
}

// Görseli Sıkıştırma
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

// Görseli Çözme
fn decode(file_path: &str) -> DynamicImage {
    let mut file = File::open(format!("{}.vfx", file_path.trim())).unwrap();
    let mut compressed_data = Vec::new();
    file.read_to_end(&mut compressed_data).unwrap();

    let raw_data = decode_with_zstd(&compressed_data);

    let data_str = String::from_utf8_lossy(&raw_data);
    let width: u32 = data_str.lines()
        .find(|line| line.contains("Width:")).unwrap()
        .split(':').nth(1).unwrap().trim().parse().unwrap();
    let height: u32 = data_str.lines()
        .find(|line| line.contains("Height:")).unwrap()
        .split(':').nth(1).unwrap().trim().parse().unwrap();

    let img_data = &raw_data[..raw_data.len() - (width.to_string().len() + height.to_string().len() + 16)];
    let mut img_pixels = Vec::new();

    for chunk in img_data.chunks(4) {
        if chunk.len() == 4 {
            img_pixels.push(Rgba([chunk[0], chunk[1], chunk[2], chunk[3]]));
        }
    }

    let img = RgbaImage::from_raw(width, height, img_pixels.into_iter()
        .flat_map(|p| p.0.to_vec())
        .collect())
        .unwrap_or_else(|| panic!("Görsel oluşturulamadı."));

    DynamicImage::ImageRgba8(img)
}

// Ana Döngü
fn main() {
    loop {
        println!("Ne yapmak istersiniz? (convert/open): ");
        let mut what_to_do = String::new();
        stdin().read_line(&mut what_to_do).unwrap();

        match what_to_do.trim() {
            "convert" => {
                println!("Görsel dosyasının yolunu girin: ");
                let mut img_path = String::new();
                stdin().read_line(&mut img_path).unwrap();

                println!("Dosyanın adını girin: ");
                let mut file_name = String::new();
                stdin().read_line(&mut file_name).unwrap();

                let img = image::open(img_path.trim()).unwrap();
                encode(img, &file_name);
                println!("Dönüştürme başarılı! '{}' kaydedildi.", file_name.trim());
            }
            "open" => {
                println!("VFX dosyasının adını ve konumunu girin: ");
                let mut vfx_path = String::new();
                stdin().read_line(&mut vfx_path).unwrap();

                let img = decode(&vfx_path);
                println!("Çözme başarılı, görsel gösteriliyor...");
                
                let (width, height) = img.dimensions();
                let rgba_image = img.to_rgba8();

                let mut buffer: Vec<u32> = Vec::with_capacity((width * height) as usize);
                for y in 0..height {
                    for x in 0..width {
                        let pixel = rgba_image.get_pixel(x, y);
                        let rgba = pixel.0;
                        let color = (u32::from(rgba[0]) << 16) | (u32::from(rgba[1]) << 8) | u32::from(rgba[2]);
                        buffer.push(color);
                    }
                }

                let mut window = Window::new(
                    "Görsel Gösterici",
                    width as usize,
                    height as usize,
                    WindowOptions {
                        resize: false,
                        scale: minifb::Scale::X1,
                        ..WindowOptions::default()
                    },
                ).unwrap_or_else(|e| panic!("Pencere açılamadı: {}", e));

                while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
                    window.update_with_buffer(&buffer, width as usize, height as usize).unwrap();
                }
            }
            _ => {
                println!("Geçersiz seçenek! Lütfen 'convert' veya 'open' girin.");
            }
        }
    }
}
