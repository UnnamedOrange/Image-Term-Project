pub mod decode_step1;
pub mod decode_step2;
pub mod encode_step1;
pub mod encode_step2;
pub mod encode_step3;
pub mod encode_step4;
pub mod encode_step5;
pub mod encode_step6;
pub mod encode_step7;

use std::io;

use image::RgbImage;

use decode_step1::decode_step1;
use decode_step2::decode_step2;
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
use encode_step6::encode_step6;
use encode_step7::encode_step7;

pub fn encode(image: &RgbImage) -> io::Result<()> {
    // 第一步：输入 RGB 的图像，输出 YUV422 的图像。
    let yuv_image = encode_step1(image)?;
    show_step1(&yuv_image);

    // 第二步：输入 YUV422 图像，输出所有 MCU。
    let mcu_collection = encode_step2(&yuv_image)?;
    show_step2(&mcu_collection);

    // 第三步：离散余弦变换。
    let dct_mcu_collection = encode_step3(&mcu_collection)?;
    show_step3(&dct_mcu_collection);

    // 第四步：量化。
    let quantized_mcu_collection = encode_step4(&dct_mcu_collection)?;
    show_step4(&quantized_mcu_collection);

    // 第五步：Zigzag。
    let zigzag_mcu_collection = encode_step5(&quantized_mcu_collection)?;
    show_step5(&zigzag_mcu_collection);

    // 第六步：编码。
    let jpeg_output_data = encode_step6(&zigzag_mcu_collection)?;

    // 第七步：输出 JPEG 文件。
    encode_step7(&jpeg_output_data)
}

pub fn decode(buf: &[u8]) -> io::Result<()> {
    let complete_jpeg_data = decode_step1(buf)?;

    let zigzag_mcu_collection = decode_step2(&complete_jpeg_data)?;

    todo!()
}
