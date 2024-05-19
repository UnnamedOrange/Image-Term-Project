use std::io;
use std::rc::Rc;

use super::decode_step1::CompleteJpegData;
use super::decode_step1::Component;
use super::encode_step4::QuantizationTable;
use super::encode_step5::ZigzagDu;

#[derive(Debug)]
pub struct DecodeZigzagMcuCollection {
    pub width: usize,
    pub height: usize,
    pub quatization_tables: Vec<Rc<QuantizationTable>>,
    pub components: Vec<Component>,
    pub zigzag_dus: Vec<ZigzagDu>,
}

/// 第二步：解码熵编码，得到一系列 Zigzag 形式的 DU。
pub fn decode_step2(jpeg_data: &CompleteJpegData) -> io::Result<DecodeZigzagMcuCollection> {
    let mut zigzag_dus = vec![];

    Ok(DecodeZigzagMcuCollection {
        width: jpeg_data.width,
        height: jpeg_data.height,
        quatization_tables: jpeg_data.quatization_tables.clone(),
        components: jpeg_data.components.clone(),
        zigzag_dus,
    })
}
