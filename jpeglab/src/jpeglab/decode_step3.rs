use std::f64::consts::PI;
use std::io;

use super::decode_step2::DecodeZigzagMcuCollection;
use super::encode_step2::Du;
use super::encode_step3::DctDu;
use super::encode_step4::QuantizationTable;
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

impl QuantizedDu {
    pub fn to_dct_du(&self, quantization_table: &QuantizationTable) -> DctDu {
        let input = &self.0;
        let qt = &quantization_table.0;
        let mut ret = [[0_f64; 8]; 8];

        for x in 0..8 {
            for y in 0..8 {
                ret[x][y] = (input[x][y] as i16 * qt[x][y] as i16) as f64;
            }
        }

        DctDu(ret)
    }
}

impl DctDu {
    pub fn idct(&self) -> Du {
        const N: usize = 8;

        let first_factor = (1.0 / N as f64).sqrt();
        let others_factor = (2.0 / N as f64).sqrt();

        let input = &self.0;
        let mut one = [[0_f64; N]; N];
        let mut ret = [[0_f64; N]; N];

        for x in 0..N {
            for y in 0..N {
                for u in 0..N {
                    one[x][y] += if u == 0 { first_factor } else { others_factor }
                        * input[u][y]
                        * (((2 * x + 1) * u) as f64 * PI / (2 * N) as f64).cos();
                }
            }
        }

        for y in 0..N {
            for x in 0..N {
                for v in 0..N {
                    ret[x][y] += if v == 0 { first_factor } else { others_factor }
                        * one[x][v]
                        * (((2 * y + 1) * v) as f64 * PI / (2 * N) as f64).cos();
                }
            }
        }

        Du(ret.map(|inner| inner.map(|it| it.round() as i8)))
    }
}

fn quantized_du_to_dus(
    decode_zigzag_mcu_collection: &DecodeZigzagMcuCollection,
    quantized_dus: &Vec<QuantizedDu>,
) -> Vec<Du> {
    let mut ret = vec![];
    let mut idx = 0;
    while idx < quantized_dus.len() {
        for component in &decode_zigzag_mcu_collection.components {
            let sf = component.horizontal_sampling_factor * component.vertical_sampling_factor;
            for _ in 0..sf {
                let quantized_du = &quantized_dus[idx];
                let dct_du = quantized_du.to_dct_du(&component.quatization_table);
                let du = dct_du.idct();
                ret.push(du);
                idx += 1;
            }
        }
    }
    ret
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
    let dus = quantized_du_to_dus(decode_zigzag_mcu_collection, &quantized_dus);

    todo!()
}

#[cfg(test)]
mod test {
    use super::*;

    use super::super::encode_step3::dct;
    use super::super::encode_step4::LUMINANCE_QUANTIZATION_TABLE;

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

    #[test]
    fn test_quantize() {
        const QUANTIZED_DU_TABLE: [[i16; 8]; 8] = [
            [-26, -3, -6, 2, 2, -1, 0, 0],
            [0, -2, -4, 1, 1, 0, 0, 0],
            [-3, 1, 5, -1, -1, 0, 0, 0],
            [-3, 1, 2, -1, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
        ];
        const DCT_DU_TABLE: [[f64; 8]; 8] = [
            [-416.0, -33.0, -60.0, 32.0, 48.0, -40.0, 0.0, 0.0],
            [0.0, -24.0, -56.0, 19.0, 26.0, 0.0, 0.0, 0.0],
            [-42.0, 13.0, 80.0, -24.0, -40.0, 0.0, 0.0, 0.0],
            [-42.0, 17.0, 44.0, -29.0, 0.0, 0.0, 0.0, 0.0],
            [18.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ];

        let quantized_du = QuantizedDu(QUANTIZED_DU_TABLE);
        let dct_du = quantized_du.to_dct_du(&LUMINANCE_QUANTIZATION_TABLE);

        assert_eq!(dct_du.0, DCT_DU_TABLE);
    }

    #[test]
    fn test_idct() {
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

        let dct_du_table = dct(&Du(DU_TABLE));
        let idct = dct_du_table.idct();

        assert_eq!(idct.0, DU_TABLE);
    }
}
