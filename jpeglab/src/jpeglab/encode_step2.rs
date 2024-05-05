use std::io;

use super::encode_step1::MyYuvImage;

/// DU 是 8x8 的有符号数。
#[derive(Debug)]
pub struct Du(pub [[i8; 8]; 8]);

/// YUV422 的 MCU，对应原始图像的 16x8 区域。
#[derive(Debug)]
pub struct Mcu {
    pub y0: Du,
    pub y1: Du,
    pub cb: Du,
    pub cr: Du,
}

#[derive(Debug)]
pub struct McuCollection {
    pub original_width: usize,
    pub original_height: usize,
    pub mcus: Vec<Mcu>,
}

/// 第二步：输入 YUV422 图像，输出所有 MCU。
/// Y0 在 Y1 的左边。
/// 无符号数转有符号数需要减去 128。
pub fn encode_step2(yuv_image: &MyYuvImage) -> io::Result<McuCollection> {
    let padded_width = yuv_image.padded_width();
    let padded_height = yuv_image.padded_height();
    let mut mcus = Vec::new();

    for y in (0..padded_height).step_by(8) {
        for x in (0..padded_width).step_by(16) {
            let mut y0 = Du([[0; 8]; 8]);
            let mut y1 = Du([[0; 8]; 8]);
            let mut cb = Du([[0; 8]; 8]);
            let mut cr = Du([[0; 8]; 8]);

            for row in 0..8 {
                for col in 0..8 {
                    let y_index = (y + row) * padded_width + (x + col);
                    y0.0[row][col] = (yuv_image.y[y_index] as i8).wrapping_add(-128);

                    let y_index = (y + row) * padded_width + (x + col + 8);
                    y1.0[row][col] = (yuv_image.y[y_index] as i8).wrapping_add(-128);

                    let u_index = (y + row) * (padded_width / 2) + (x / 2 + col);
                    cb.0[row][col] = (yuv_image.u[u_index] as i8).wrapping_add(-128);

                    let v_index = (y + row) * (padded_width / 2) + (x / 2 + col);
                    cr.0[row][col] = (yuv_image.v[v_index] as i8).wrapping_add(-128);
                }
            }

            mcus.push(Mcu { y0, y1, cb, cr });
        }
    }

    Ok(McuCollection {
        original_width: yuv_image.original_width,
        original_height: yuv_image.original_height,
        mcus,
    })
}

pub fn show_step2(result: &McuCollection) {
    println!(
        "[INFO] 大小为 {}x{} 的 RGB 图像编码出 {} 个 MCU，共 {} 个 DU",
        result.original_width,
        result.original_height,
        result.mcus.len(),
        result.mcus.len() * 4,
    );
    println!("[VERBOSE] MCU 的例子：\n{:?}", &result.mcus[0]);
}
