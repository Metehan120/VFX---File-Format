use crate::lib::decoder_old;
use crate::lib::encoder;

pub fn update(file_path: &String) {
    let image = decoder_old::decode(file_path);

    encoder::encode(image, &file_path);
}