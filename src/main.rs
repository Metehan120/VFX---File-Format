use std::io::{stdin, Write};
use std::fs::File;
use std::string;
use image::{self, DynamicImage, GenericImageView};
use minifb::{Window, WindowOptions};
use zstd::stream::Encoder;
use lib::{decoder, updater};
mod lib {
    pub mod decoder;
    pub mod decoder_old;
    pub mod updater;
}

fn encode_with_zstd(input_data: &[u8]) -> Vec<u8> {
    println!("Sıkıştırma öncesi boyut: {}", input_data.len());
    let mut compressed_data = Vec::new();
    let mut encoder = Encoder::new(&mut compressed_data, 11).expect("Sıkıştırıcı başlatılamadı");
    encoder.write_all(input_data).expect("Sıkıştırma hatası");
    encoder.finish().expect("Sıkıştırıcı bitirme hatası");
    println!("Sıkıştırma sonrası boyut: {}", compressed_data.len());
    compressed_data
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

    let height_hex = hex::encode(format!("Height: {}", height));
    let width_hex = hex::encode(format!("Width: {}", width));

    let info = format!("\n{}\n{}\n{}", height_hex, width_hex, signature);
    img_data.extend_from_slice(info.as_bytes());

    let compressed_data = encode_with_zstd(&img_data);

    let mut file = match File::create(format!("{}.vfx", file_name.trim())) {
        Ok(f) => f,
        Err(e) => panic!("Dosya açılırken hata oluştu: {}", e),
    };

    if let Err(e) = file.write_all(&compressed_data) {
        panic!("Yazma hatası: {}", e);
    }
}

fn update(file_path: &string::String) {
    updater::update(&file_path);
}

// Ana Döngü
fn main() {
    loop {
        println!("Versiyon: 3");
        println!("Ne yapmak istersiniz? (convert/open/update (versiyon 1 içindir): ");
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

                let img = decoder::decode(&vfx_path);
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
            "update" => {
                println!("VFX dosyasının adını ve konumunu girin: ");
                let mut vfx_path = String::new();
                stdin().read_line(&mut vfx_path).unwrap();

                update(&vfx_path);
                println!("Güncelleme başarılı! '{}' güncellendi.", vfx_path.trim());
            }

            _ => {
                println!("Geçersiz seçenek! Lütfen 'convert', 'open' veya 'update' girin.");
            }
        }
    }
}
