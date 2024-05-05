pub mod encode_step1;
pub mod encode_step2;

use std::io;

use image::RgbImage;

use encode_step1::encode_step1;
use encode_step1::show_step1;
use encode_step2::encode_step2;
use encode_step2::show_step2;

pub fn encode(image: &RgbImage) -> io::Result<()> {
    let yuv_image = encode_step1(image)?;
    show_step1(&yuv_image);

    let mcu_collections = encode_step2(&yuv_image)?;
    show_step2(&mcu_collections);

    todo!()
}
