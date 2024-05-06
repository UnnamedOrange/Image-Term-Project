pub mod encode_step1;
pub mod encode_step2;
pub mod encode_step3;
pub mod encode_step4;

use std::io;

use image::RgbImage;

use encode_step1::encode_step1;
use encode_step1::show_step1;
use encode_step2::encode_step2;
use encode_step2::show_step2;
use encode_step3::encode_step3;
use encode_step3::show_step3;
use encode_step4::encode_step4;
use encode_step4::show_step4;

pub fn encode(image: &RgbImage) -> io::Result<()> {
    let yuv_image = encode_step1(image)?;
    show_step1(&yuv_image);

    let mcu_collections = encode_step2(&yuv_image)?;
    show_step2(&mcu_collections);

    let dct_mcu_collections = encode_step3(&mcu_collections)?;
    show_step3(&dct_mcu_collections);

    let quantized_mcu_collections = encode_step4(&dct_mcu_collections)?;
    show_step4(&quantized_mcu_collections);

    todo!()
}
