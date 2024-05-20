use std::io;

use super::encode_step4::QuantizedDu;
use super::encode_step4::QuantizedMcuCollection;

/// Zigzag 后的 DU。
#[derive(Debug)]
pub struct ZigzagDu(pub [i16; 64]);

/// Zigzag 后的 MCU。
#[derive(Debug)]
pub struct ZigzagMcu {
    pub y0: ZigzagDu,
    pub y1: ZigzagDu,
    pub cb: ZigzagDu,
    pub cr: ZigzagDu,
}

#[derive(Debug)]
pub struct ZigzagMcuCollection {
    pub original_width: usize,
    pub original_height: usize,
    pub zigzag_mcus: Vec<ZigzagMcu>,
}

impl QuantizedDu {
    pub fn zigzag(&self) -> ZigzagDu {
        let input = &self.0;
        let mut ret = [0_i16; 64];

        let mut x = 0;
        let mut y = 0;
        let mut idx = 0;

        while idx < ret.len() {
            while idx < ret.len() {
                ret[idx] = input[x][y];
                idx += 1;
                // 优先处理 y == 7，因为有对角线。
                if y == 7 {
                    x += 1;
                    break;
                } else if x == 0 {
                    y += 1;
                    break;
                } else {
                    x -= 1;
                    y += 1;
                }
            }
            while idx < ret.len() {
                ret[idx] = input[x][y];
                idx += 1;
                // 优先处理 x == 7，因为有对角线。
                if x == 7 {
                    y += 1;
                    break;
                } else if y == 0 {
                    x += 1;
                    break;
                } else {
                    x += 1;
                    y -= 1;
                }
            }
        }

        ZigzagDu(ret)
    }
}

/// 第五步：Zigzag。
pub fn encode_step5(
    quantized_mcu_collection: &QuantizedMcuCollection,
) -> io::Result<ZigzagMcuCollection> {
    let mut zigzag_mcus = Vec::new();

    for mcu in &quantized_mcu_collection.quantized_mcus {
        zigzag_mcus.push(ZigzagMcu {
            y0: mcu.y0.zigzag(),
            y1: mcu.y1.zigzag(),
            cb: mcu.cb.zigzag(),
            cr: mcu.cr.zigzag(),
        });
    }

    Ok(ZigzagMcuCollection {
        original_width: quantized_mcu_collection.original_width,
        original_height: quantized_mcu_collection.original_height,
        zigzag_mcus,
    })
}

pub fn show_step5(result: &ZigzagMcuCollection) {
    println!("[VERBOSE] Zigzag 的例子：\n{:?}", &result.zigzag_mcus[0]);
}

#[cfg(test)]
mod test {
    use super::super::encode_step4::QuantizedDu;

    #[test]
    fn test_zigzag() {
        const DU_TABLE: [[i16; 8]; 8] = [
            [0, 1, 5, 6, 14, 15, 27, 28],
            [2, 4, 7, 13, 16, 26, 29, 42],
            [3, 8, 12, 17, 25, 30, 41, 43],
            [9, 11, 18, 24, 31, 40, 44, 53],
            [10, 19, 23, 32, 39, 45, 52, 54],
            [20, 22, 33, 38, 46, 51, 55, 60],
            [21, 34, 37, 47, 50, 56, 59, 61],
            [35, 36, 48, 49, 57, 58, 62, 63],
        ];
        const ZIGZAG_DU_TABLE: [i16; 64] = [
            0, 1, 2, 3, 4, 5, 6, 7, //
            8, 9, 10, 11, 12, 13, 14, 15, //
            16, 17, 18, 19, 20, 21, 22, 23, //
            24, 25, 26, 27, 28, 29, 30, 31, //
            32, 33, 34, 35, 36, 37, 38, 39, //
            40, 41, 42, 43, 44, 45, 46, 47, //
            48, 49, 50, 51, 52, 53, 54, 55, //
            56, 57, 58, 59, 60, 61, 62, 63, //
        ];

        let quantized_du = QuantizedDu(DU_TABLE);
        let zigzag_du = quantized_du.zigzag();

        assert_eq!(zigzag_du.0, ZIGZAG_DU_TABLE);
    }
}
