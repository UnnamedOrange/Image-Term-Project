use std::io;

use bitvec::vec::BitVec;

use super::encode_step4::QuantizationTable;
use super::encode_step6::CachedHuffmanTable;

/// 分量信息。来源于 SOF0 和 SOS。
#[derive(Debug)]
pub struct Component<'a> {
    /// 相对水平采样因子。
    pub horizontal_sampling_factor: u8,
    /// 相对垂直采样因子。
    pub vertical_sampling_factor: u8,
    /// 量化表。
    pub quatization_table: &'a QuantizationTable,
    /// DC 霍夫曼表。
    pub dc_huffman_table: &'a CachedHuffmanTable,
    /// AC 霍夫曼表。
    pub ac_huffman_table: &'a CachedHuffmanTable,
}

/// 解码 JPEG 图像所需的完整数据，使用方便编程的格式。
#[derive(Debug)]
pub struct CompleteJpegData<'a> {
    /// 图像宽度，列数。
    pub width: usize,
    /// 图像高度，行数。
    pub height: usize,
    /// 量化表，总是为 16 位精度，ID 为下标。
    pub quatization_tables: Vec<QuantizationTable>,
    /// 霍夫曼表，ID 为下标。
    pub huffman_tables: Vec<CachedHuffmanTable>,
    /// 分量信息。
    pub components: Vec<Component<'a>>,
    /// 图像数据。
    pub scan: BitVec,
}

/// 第一步：从原始的 JPEG 数据中解析出解码所需的完整数据。
pub fn decode_step1(buf: &[u8]) -> io::Result<CompleteJpegData> {
    todo!()
}
