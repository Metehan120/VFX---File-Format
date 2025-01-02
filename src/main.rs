use std::{fmt::Error, io::Write};
use std::default::Default;
use std::fs::File;
use std::string;
use image::{self, DynamicImage, GenericImageView};
use zstd::stream::Encoder;
use lib::{decoder, updater};
use eframe::egui;
use tinyfiledialogs;
use winapi;

mod lib {
    pub mod decoder;
    pub mod decoder_old;
    pub mod updater;
}

fn encode_with_zstd(input_data: &[u8]) -> Vec<u8> {
    let mut compressed_data = Vec::new();
    let mut encoder = Encoder::new(&mut compressed_data, 11).expect("Failed to initialize compressor");
    encoder.write_all(input_data).expect("Compression error");
    encoder.finish().expect("Failed to finalize compressor");
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

fn main() -> Result<(), Error> {
    unsafe { winapi::um::wincon::FreeConsole() };

    let mut native_options = eframe::NativeOptions::default();
    native_options.initial_window_size = Some(egui::vec2(500.0, 500.0));

    eframe::run_native(
        "VFX Editor (Version 3)",
        native_options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

#[derive(Default)]
struct MyApp {
    file_name: string::String,
    img: DynamicImage
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("File Name: (No need for VFX files)");
                ui.text_edit_singleline(&mut self.file_name);
            });
            
            ui.horizontal(|ui| {
                if ui.button("Convert").clicked() {
                    if let Some(path) = tinyfiledialogs::open_file_dialog("Open Image", "", None) {
                        match image::open(&path) {
                            Ok(img) => { 
                                encode(img.clone(), &self.file_name);
                                ui.label("File encoded successfully!");
                            }
                            Err(e) => {
                                ui.label(format!("Failed to open image: {}", e));
                            }
                        }
                    }
                }

                if ui.button("Open").clicked() {
                    if let Some(path) = tinyfiledialogs::open_file_dialog("Open Image", "", None) {
                        self.img = decoder::decode(&path);
                    }
                }

                if ui.button("Update (For Version 1)").clicked() {
                    if let Some(path) = tinyfiledialogs::open_file_dialog("Open Image", "", None) {
                        updater::update(&path);
                    }
                }
            });

            ui.separator();

            if self.img.dimensions() != (0, 0) {
                let (width, height) = self.img.dimensions();
                let available_size = ui.available_size();
                let aspect_ratio = width as f32 / height as f32;
                let display_size = if available_size.x / aspect_ratio <= available_size.y {
                    [available_size.x, available_size.x / aspect_ratio]
                } else {
                    [available_size.y * aspect_ratio, available_size.y]
                };

                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    [width as usize, height as usize],
                    &self.img.to_rgba8(),
                );
                let texture_id = ctx.load_texture("image", color_image);
                ui.image(&texture_id, display_size);
            }
        });
    }
}