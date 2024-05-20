use std::io;

use image::ImageBuffer;
use image::ImageFormat;

use super::decode_step3::DecodedYuvImage;
use super::encode_step1::yuv_to_rgb;

/// 第四步：将 YUV 转换为 RGB，输出 BMP 文件。
/// 文件名为 out.bmp。
pub fn decode_step4(decoded_yuv_image: &DecodedYuvImage) -> io::Result<()> {
    // 使用外部库完成输出 BMP。
    let mut img = ImageBuffer::new(
        decoded_yuv_image.width as u32,
        decoded_yuv_image.height as u32,
    );

    let max_h = *[
        decoded_yuv_image.y.absolute_horizontal_sampling_factor,
        decoded_yuv_image.u.absolute_horizontal_sampling_factor,
        decoded_yuv_image.v.absolute_horizontal_sampling_factor,
    ]
    .iter()
    .max()
    .unwrap();
    let hb = 8 * max_h;
    let padded_width = (decoded_yuv_image.width + hb - 1) / hb * hb;

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let x = x as usize;
        let y = y as usize;

        let c = &decoded_yuv_image.y;
        let hs = c.absolute_horizontal_sampling_factor;
        let vs = c.absolute_vertical_sampling_factor;
        let yc = y / vs;
        let xc = x / hs;
        let y_ = c.values[yc * padded_width / hs + xc];

        let c = &decoded_yuv_image.u;
        let hs = c.absolute_horizontal_sampling_factor;
        let vs = c.absolute_vertical_sampling_factor;
        let yc = y / vs;
        let xc = x / hs;
        let u = c.values[yc * padded_width / hs + xc];

        let c = &decoded_yuv_image.v;
        let hs = c.absolute_horizontal_sampling_factor;
        let vs = c.absolute_vertical_sampling_factor;
        let yc = y / vs;
        let xc = x / hs;
        let v = c.values[yc * padded_width / hs + xc];

        let (r, g, b) = yuv_to_rgb(y_, u, v);
        *pixel = image::Rgb([r, g, b]);
    }

    img.save_with_format("out.bmp", ImageFormat::Bmp)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Fail to write to BMP file"))?;

    Ok(())
}
