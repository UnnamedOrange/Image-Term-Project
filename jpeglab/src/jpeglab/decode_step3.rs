use std::io;

use super::decode_step2::DecodeZigzagMcuCollection;
use super::encode_step4::QuantizedDu;
use super::encode_step5::ZigzagDu;

#[derive(Debug)]
pub struct YuvComponent {
    pub absolute_horizontal_sampling_factor: usize,
    pub absolute_vertical_sampling_factor: usize,
    pub values: Vec<u8>,
}

#[derive(Debug)]
pub struct DecodedYuvImage {
    pub width: usize,
    pub height: usize,
    pub y: YuvComponent,
    pub u: YuvComponent,
    pub v: YuvComponent,
}

impl ZigzagDu {
    pub fn to_quantized_du(&self) -> QuantizedDu {
        let input = &self.0;
        let mut ret = [[0; 8]; 8];

        let mut x = 0;
        let mut y = 0;
        let mut idx = 0;

        while idx < input.len() {
            while idx < input.len() {
                ret[x][y] = input[idx];
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
            while idx < input.len() {
                ret[x][y] = input[idx];
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

        QuantizedDu(ret)
    }
}

/// 第三步：直接解码为填充的 YUV 图像。
pub fn decode_step3(
    decode_zigzag_mcu_collection: &DecodeZigzagMcuCollection,
) -> io::Result<DecodedYuvImage> {
    let quantized_dus: Vec<QuantizedDu> = decode_zigzag_mcu_collection
        .zigzag_dus
        .iter()
        .map(|it| it.to_quantized_du())
        .collect();

    todo!()
}

#[cfg(test)]
mod test {
    use super::*;

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

        let zigzag_du = ZigzagDu(ZIGZAG_DU_TABLE);
        let quantized_du = zigzag_du.to_quantized_du();

        assert_eq!(quantized_du.0, DU_TABLE);
    }
}
