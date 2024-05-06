use std::io;

use super::encode_step3::DctDu;
use super::encode_step3::DctMcuCollection;

/// 量化后的 DU。
/// 根据系数的编码表，设定为 16 位有符号整数。
#[derive(Debug)]
pub struct QuantizedDu(pub [[i16; 8]; 8]);

/// 量化表。
/// 根据量化后的 DU，设定为 16 位无符号整数。
#[derive(Debug)]
pub struct QuantizationTable(pub [[u16; 8]; 8]);

/// 亮度量化表。
pub const LUMINANCE_QUANTIZATION_TABLE: QuantizationTable = QuantizationTable([
    [16, 11, 10, 16, 24, 40, 51, 61],
    [12, 12, 14, 19, 26, 58, 60, 55],
    [14, 13, 16, 24, 40, 57, 69, 56],
    [14, 17, 22, 29, 51, 87, 80, 62],
    [18, 22, 37, 56, 68, 109, 103, 77],
    [24, 35, 55, 64, 81, 104, 113, 92],
    [49, 64, 78, 87, 103, 121, 120, 101],
    [72, 92, 95, 98, 112, 100, 103, 99],
]);

/// 色度量化表。
pub const CHROMINANCE_QUANTIZATION_TABLE: QuantizationTable = QuantizationTable([
    [17, 18, 24, 47, 99, 99, 99, 99],
    [18, 21, 26, 66, 99, 99, 99, 99],
    [24, 26, 56, 99, 99, 99, 99, 99],
    [47, 66, 99, 99, 99, 99, 99, 99],
    [99, 99, 99, 99, 99, 99, 99, 99],
    [99, 99, 99, 99, 99, 99, 99, 99],
    [99, 99, 99, 99, 99, 99, 99, 99],
    [99, 99, 99, 99, 99, 99, 99, 99],
]);

impl DctDu {
    pub fn quantize(&self, table: &QuantizationTable) -> QuantizedDu {
        let mut ret = [[0_i16; 8]; 8];

        for i in 0..8 {
            for j in 0..8 {
                ret[i][j] = (self.0[i][j] / table.0[i][j] as f64).round() as i16;
            }
        }

        QuantizedDu(ret)
    }
}

/// 量化后的 MCU。
#[derive(Debug)]
pub struct QuantizedMcu {
    pub y0: QuantizedDu,
    pub y1: QuantizedDu,
    pub cb: QuantizedDu,
    pub cr: QuantizedDu,
}

#[derive(Debug)]
pub struct QuantizedMcuCollection {
    pub original_width: usize,
    pub original_height: usize,
    pub quantized_mcus: Vec<QuantizedMcu>,
}

/// 第四步：量化。
pub fn encode_step4(dct_mcu_collection: &DctMcuCollection) -> io::Result<QuantizedMcuCollection> {
    let mut quantized_mcus = Vec::new();

    for mcu in &dct_mcu_collection.dct_mcus {
        quantized_mcus.push(QuantizedMcu {
            y0: mcu.y0.quantize(&LUMINANCE_QUANTIZATION_TABLE),
            y1: mcu.y1.quantize(&LUMINANCE_QUANTIZATION_TABLE),
            cb: mcu.cb.quantize(&CHROMINANCE_QUANTIZATION_TABLE),
            cr: mcu.cr.quantize(&CHROMINANCE_QUANTIZATION_TABLE),
        });
    }

    Ok(QuantizedMcuCollection {
        original_width: dct_mcu_collection.original_width,
        original_height: dct_mcu_collection.original_height,
        quantized_mcus,
    })
}

pub fn show_step4(result: &QuantizedMcuCollection) {
    println!("[VERBOSE] 量化的例子：\n{:?}", &result.quantized_mcus[0]);
}

#[cfg(test)]
mod test {
    use super::*;

    use super::super::encode_step2::Du;
    use super::super::encode_step3::dct;

    #[test]
    fn test_quantize() {
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

        let dct_du = dct(&Du(DU_TABLE));
        let quantized_du = dct_du.quantize(&LUMINANCE_QUANTIZATION_TABLE);

        assert_eq!(quantized_du.0, QUANTIZED_DU_TABLE);
    }
}
