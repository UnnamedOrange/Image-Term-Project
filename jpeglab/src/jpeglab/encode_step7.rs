use std::io;
use std::path::Path;

use super::encode_step6::JpegOutputData;

/// 图像开始。
/// FF D8
#[derive(Debug)]
pub struct SOI;

/// 应用程序保留标记 0。
/// FF E0
#[derive(Debug)]
pub struct APP0 {
    /// 块长度（不含起始符号 FF E0）。总是为 16。
    pub length: u16,
    pub identifier: [u8; 5],
    pub major_version: u8,
    pub minor_version: u8,
    pub units: u8,
    pub x_density: u16,
    pub y_density: u16,
    pub x_thumbnail: u8,
    pub y_thumbnail: u8,
}

impl Default for APP0 {
    fn default() -> Self {
        Self {
            length: 16,
            identifier: *b"JFIF\0",
            major_version: 1,
            minor_version: 1,
            units: 0, // NoUnits.
            x_density: 1,
            y_density: 1,
            x_thumbnail: 0,
            y_thumbnail: 0,
        }
    }
}

/// 量化表。
/// FF DB
#[derive(Debug)]
pub struct DQT {
    /// 块长度（不含起始符号 FF DB）。输出为 131。
    pub length: u16,
    /// 量化表的精度，在原始结构中占 1 个字节的高 4 位。注意是完全大端。
    /// 1 表示 16 位，0 表示 8 位。
    pub is_precision_16: bool,
    /// 量化表的 ID，在原始结构占 1 个字节的低 4 位。
    /// 取值范围是 0 到 3。
    pub id: u8,
    /// 量化表的值。默认以 16 位精度存储。
    pub table: [u16; 64],
}

impl Default for DQT {
    fn default() -> Self {
        Self {
            length: 131,
            is_precision_16: true,
            id: 0,
            table: [0; 64],
        }
    }
}

/// SOF0 中用到的分量信息。
#[derive(Debug)]
pub struct SOF0Component {
    /// 分量的 ID，通常取 1, 2, 3。
    pub id: u8,
    /// 水平采样因子，在原始结构中占 1 个字节的高 4 位。
    pub horizontal_sampling_factor: u8,
    /// 垂直采样因子，在原始结构中占 1 个字节的低 4 位。
    pub vertical_sampling_factor: u8,
    /// 采用的量化表的编号。
    pub quantization_id: u8,
}

/// 帧图像开始标记 0。
/// FF C0
#[derive(Debug)]
pub struct SOF0 {
    /// 块长度（不含起始符号 FF C0）。总是为 17。
    pub length: u16,
    /// 每个颜色分量的位数。只支持 8。
    pub precision: u8,
    /// 行数，高。
    pub lines: u16,
    /// 每行的采样点数，宽。
    pub samples_per_line: u16,
    /// 各个分量。分量数蕴含在其中。
    pub components: Vec<SOF0Component>,
}

impl Default for SOF0 {
    fn default() -> Self {
        Self {
            length: 17,
            precision: 8,
            lines: 0,
            samples_per_line: 0,
            components: vec![
                SOF0Component {
                    id: 1,
                    horizontal_sampling_factor: 2,
                    vertical_sampling_factor: 1,
                    quantization_id: 0,
                },
                SOF0Component {
                    id: 2,
                    horizontal_sampling_factor: 1,
                    vertical_sampling_factor: 1,
                    quantization_id: 1,
                },
                SOF0Component {
                    id: 3,
                    horizontal_sampling_factor: 1,
                    vertical_sampling_factor: 1,
                    quantization_id: 1,
                },
            ],
        }
    }
}

/// 霍夫曼表。
/// FF C4
#[derive(Debug)]
pub struct DHT {
    /// 块长度（不含起始符号 FF C4）。
    pub length: u16,
    /// 霍夫曼表的类别，在原始结构中占 1 个字节的高 4 位。
    /// 0 表示 DC，1 表示 AC。
    pub table_class: u8,
    /// 霍夫曼表的编号，在原始结构中占 1 个字节的低 4 位。
    pub id: u8,
    /// 位表。即编码长度为 `i + 1` 的符号数目。
    pub codes: [u8; 16],
    /// 值表。
    pub values: Vec<u8>,
}

/// SOS 中用到的分量信息。
#[derive(Debug)]
pub struct SOSComponent {
    /// 分量的 ID，通常取 1, 2, 3。
    pub id: u8,
    /// DC 编码的霍夫曼表 ID，在原始结构中占 1 个字节的高 4 位。
    pub dc_huffman_id: u8,
    /// AC 编码的霍夫曼表 ID，在原始结构中占 1 个字节的低 4 位。
    pub ac_huffman_id: u8,
}

/// 扫描开始标记。
/// FF DA
#[derive(Debug)]
pub struct SOS {
    /// 块长度（不含起始符号 FF DA）。总是为 12。
    pub length: u16,
    /// 各个分量。分量数蕴含在其中。
    pub components: Vec<SOSComponent>,
    /// Spectral Selection Start。频谱选择的开始系数。总是为 0。
    pub ss: u8,
    /// Spectral Selection End。频谱选择的结束系数。总是为 63。
    pub se: u8,
    /// Successive Approximation Bit Position High。逐次逼近高位，
    /// 在原始结构中占 1 个字节的高 4 位。总是为 0。
    pub ah: u8,
    /// Successive Approximation Bit Position Low。逐次逼近低位，
    /// 在原始结构中占 1 个字节的低 4 位。总是为 0。
    pub al: u8,
}

impl Default for SOS {
    fn default() -> Self {
        Self {
            length: 12,
            components: vec![
                SOSComponent {
                    id: 1,
                    dc_huffman_id: 0,
                    ac_huffman_id: 1,
                },
                SOSComponent {
                    id: 2,
                    dc_huffman_id: 2,
                    ac_huffman_id: 3,
                },
                SOSComponent {
                    id: 3,
                    dc_huffman_id: 2,
                    ac_huffman_id: 3,
                },
            ],
            ss: 0,
            se: 63,
            ah: 0,
            al: 0,
        }
    }
}

/// 图像数据。
/// 没有开始符号，只有结束符号 EOI。
#[derive(Debug)]
pub struct ImageData {
    pub data: Vec<u8>,
}

impl ImageData {
    fn new() -> Self {
        Self { data: vec![] }
    }
}

/// 图像结束。
/// FF D9
#[derive(Debug)]
pub struct EOI;

/// 第七步：输出 JPEG 文件。
/// 文件名为 out.jpg。
pub fn encode_step7(data: &JpegOutputData) -> io::Result<()> {
    let out_path = Path::new("out.jpg");

    todo!()
}
