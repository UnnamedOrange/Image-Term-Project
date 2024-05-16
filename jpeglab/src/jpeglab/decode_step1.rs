use std::convert::TryInto;
use std::io;

use bitvec::vec::BitVec;
use bytebuffer::ByteBuffer;
use bytebuffer::Endian;

use super::encode_step4::QuantizationTable;
use super::encode_step6::CachedHuffmanTable;
use super::encode_step7::APP0;

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

impl<'a> Default for CompleteJpegData<'a> {
    fn default() -> Self {
        Self {
            width: Default::default(),
            height: Default::default(),
            quatization_tables: Default::default(),
            huffman_tables: Default::default(),
            components: Default::default(),
            scan: Default::default(),
        }
    }
}

fn parse_app0(block: &[u8]) -> io::Result<APP0> {
    let mut buf = ByteBuffer::from_bytes(block);
    let mut ret = APP0::default();
    ret.length = block.len() as u16 + 2;
    ret.identifier = buf.read_bytes(5)?.try_into().unwrap();
    ret.major_version = buf.read_u8()?;
    ret.minor_version = buf.read_u8()?;
    ret.units = buf.read_u8()?;
    ret.x_density = buf.read_u16()?;
    ret.y_density = buf.read_u16()?;
    ret.x_thumbnail = buf.read_u8()?;
    ret.y_thumbnail = buf.read_u8()?;

    Ok(ret)
}

fn parse_dqt(block: &[u8]) -> io::Result<QuantizationTable> {
    let mut buf = ByteBuffer::from_bytes(block);
    let mut ret = QuantizationTable(Default::default());
    let precision_and_id = buf.read_u8()?;
    let _id = precision_and_id & 0x0F; // 忽略 ID，假设按顺序。
    let precision = precision_and_id >> 4;

    for i in 0..8 {
        for j in 0..8 {
            ret.0[i][j] = if precision == 0 {
                buf.read_u8()? as u16
            } else {
                buf.read_u16()? as u16
            };
        }
    }

    Ok(ret)
}

/// 第一步：从原始的 JPEG 数据中解析出解码所需的完整数据。
pub fn decode_step1(buf: &[u8]) -> io::Result<CompleteJpegData> {
    let mut ret = CompleteJpegData::default();
    let mut quantization_tables = vec![];

    let mut buf = ByteBuffer::from_bytes(buf);
    buf.set_endian(Endian::BigEndian);
    while buf.get_rpos() < buf.len() {
        let heading = buf.read_u8().and_then(|v| {
            if v != 0xFF {
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid block heading",
                ))
            } else {
                Ok(v)
            }
        })?;
        let block_type = buf.read_u8()?;

        match block_type {
            // SOI
            0xD8 => {}
            // APP0
            0xE0 => {
                let length = buf.read_u16()?;
                if length < 2 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid block length",
                    ));
                }
                let length = length as usize - 2;
                let block = buf.read_bytes(length)?;
                let _app0 = parse_app0(&block)?; // 不使用。
            }
            // APPn
            0xE1..=0xEF => {
                let length = buf.read_u16()?;
                if length < 2 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid block length",
                    ));
                }
                let length = length as usize - 2;
                let _block = buf.read_bytes(length)?; // 不处理。
            }
            // DQT
            0xDB => {
                let length = buf.read_u16()?;
                if length < 2 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid block length",
                    ));
                }
                let length = length as usize - 2;
                let block = buf.read_bytes(length)?;
                let dqt = parse_dqt(&block)?;
                quantization_tables.push(dqt);
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid block type",
                ));
            }
        }
    }

    todo!()
}
