pub mod encode_step1;

use std::io;

use image::RgbImage;

use encode_step1::encode_step1;
use encode_step1::show_step1;

pub fn encode(image: &RgbImage) -> io::Result<()> {
    let yuv_image = encode_step1(image)?;
    show_step1(&yuv_image);

    todo!()
}
