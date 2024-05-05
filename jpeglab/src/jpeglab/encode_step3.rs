use std::f64::consts::PI;
use std::io;

use super::encode_step2::Du;
use super::encode_step2::McuCollection;

/// DCT 后的 DU。
#[derive(Debug)]
pub struct DctDu(pub [[f64; 8]; 8]);

/// DCT 后的 MCU。
#[derive(Debug)]
pub struct DctMcu {
    pub y0: DctDu,
    pub y1: DctDu,
    pub cb: DctDu,
    pub cr: DctDu,
}

#[derive(Debug)]
pub struct DctMcuCollection {
    pub original_width: usize,
    pub original_height: usize,
    pub dct_mcus: Vec<DctMcu>,
}

pub(super) fn dct(du: &Du) -> DctDu {
    const N: usize = 8;

    let first_factor = (1.0 / N as f64).sqrt();
    let others_factor = (2.0 / N as f64).sqrt();

    let input = &du.0;
    let mut one = [[0f64; N]; N];
    let mut ret = [[0f64; N]; N];

    for u in 0..N {
        for v in 0..N {
            for y in 0..N {
                one[u][v] +=
                    input[u][y] as f64 * (((2 * y + 1) * v) as f64 * PI / ((2 * N) as f64)).cos();
            }
            one[u][v] *= if v == 0 { first_factor } else { others_factor };
        }
    }

    for v in 0..N {
        for u in 0..N {
            for x in 0..N {
                ret[u][v] +=
                    one[x][v] as f64 * (((2 * x + 1) * u) as f64 * PI / ((2 * N) as f64)).cos();
            }
            ret[u][v] *= if u == 0 { first_factor } else { others_factor };
        }
    }

    DctDu(ret)
}

/// 第三步：离散余弦变换。
pub fn encode_step3(yuv_image: &McuCollection) -> io::Result<DctMcuCollection> {
    let mut dct_mcus = Vec::new();

    for mcu in &yuv_image.mcus {
        dct_mcus.push(DctMcu {
            y0: dct(&mcu.y0),
            y1: dct(&mcu.y1),
            cb: dct(&mcu.cb),
            cr: dct(&mcu.cr),
        });
    }

    Ok(DctMcuCollection {
        original_width: yuv_image.original_width,
        original_height: yuv_image.original_height,
        dct_mcus,
    })
}

pub fn show_step3(result: &DctMcuCollection) {
    println!("[VERBOSE] MCU 计算 DCT 的例子：\n{:?}", &result.dct_mcus[0]);
}

#[cfg(test)]
mod test {
    use super::*;

    use super::super::encode_step2::Du;

    #[test]
    fn test_dct() {
        // https://blog.csdn.net/weixin_44874766/article/details/117444843
        const DU_TABLE: [[i8; 8]; 8] = [
            [-76, -73, -67, -62, -58, -67, -64, -55],
            [-65, -69, -73, -38, -19, -43, -59, -56],
            [-66, -69, -60, -15, 16, -24, -62, -55],
            [-65, -70, -57, -6, 26, -22, -58, -59],
            [-61, -67, -60, -24, -2, -40, -60, -58],
            [-49, -63, -68, -58, -51, -60, -70, -53],
            [-43, -57, -64, -69, -73, -67, -63, -45],
            [-41, -49, -59, -60, -63, -52, -50, -34],
        ];
        const DCT_DU_TABLE: [[i32; 8]; 8] = [
            [-415, -30, -61, 27, 56, -20, -2, 0],
            [4, -22, -61, 10, 13, -7, -9, 5],
            [-47, 7, 77, -25, -29, 10, 5, -6],
            [-49, 12, 34, -15, -10, 6, 2, 2],
            [12, -7, -13, -4, -2, 2, -3, 3],
            [-8, 3, 2, -6, -2, 1, 4, 2],
            [-1, 0, 0, -2, -1, -3, 4, -1],
            [0, 0, -1, -4, -1, 0, 1, 2],
        ];

        let dct_du = dct(&Du(DU_TABLE));
        let mut output = [[0i32; 8]; 8];
        for i in 0..8 {
            for j in 0..8 {
                output[i][j] = dct_du.0[i][j].round() as i32;
            }
        }

        assert_eq!(output, DCT_DU_TABLE);
    }
}
