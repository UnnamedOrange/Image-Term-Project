pub mod encode_step1;
pub mod encode_step2;
pub mod encode_step3;
pub mod encode_step4;
pub mod encode_step5;

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
use encode_step5::encode_step5;
use encode_step5::show_step5;

pub fn encode(image: &RgbImage) -> io::Result<()> {
    let yuv_image = encode_step1(image)?;
    show_step1(&yuv_image);

    let mcu_collection = encode_step2(&yuv_image)?;
    show_step2(&mcu_collection);

    let dct_mcu_collection = encode_step3(&mcu_collection)?;
    show_step3(&dct_mcu_collection);

    let quantized_mcu_collection = encode_step4(&dct_mcu_collection)?;
    show_step4(&quantized_mcu_collection);

    let zigzag_mcu_collection = encode_step5(&quantized_mcu_collection)?;
    show_step5(&zigzag_mcu_collection);

    todo!()
}
