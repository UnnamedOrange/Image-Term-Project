use std::collections::BTreeMap;
use std::convert::TryInto;
use std::io;
use std::rc::Rc;

use bitvec::order::Lsb0;
use bitvec::vec::BitVec;
use bitvec::view::BitView;
use bytebuffer::ByteBuffer;
use bytebuffer::Endian;

use super::encode_step4::QuantizationTable;
use super::encode_step6::CachedHuffmanTable;
use super::encode_step6::JpegHuffmanTable;
use super::encode_step7::APP0;

/// 分量信息。来源于 SOF0 和 SOS。
#[derive(Debug, Clone)]
pub struct Component {
    /// 相对水平采样因子。
    pub horizontal_sampling_factor: u8,
    /// 相对垂直采样因子。
    pub vertical_sampling_factor: u8,
    /// 量化表。
    pub quatization_table: Rc<QuantizationTable>,
    /// DC 霍夫曼表。
    pub dc_huffman_table: Rc<CachedHuffmanTable>,
    /// AC 霍夫曼表。
    pub ac_huffman_table: Rc<CachedHuffmanTable>,
}

/// 临时分量信息。
#[derive(Debug)]
struct TempComponent {
    pub horizontal_sampling_factor: u8,
    pub vertical_sampling_factor: u8,
    pub quatization_table_id: u8,
    pub dc_huffman_table_id: u8,
    pub ac_huffman_table_id: u8,
}

/// 解码 JPEG 图像所需的完整数据，使用方便编程的格式。
#[derive(Debug)]
pub struct CompleteJpegData {
    /// 图像宽度，列数。
    pub width: usize,
    /// 图像高度，行数。
    pub height: usize,
    /// 分量信息。
    pub components: Vec<Component>,
    /// 图像数据。
    pub scan: BitVec,
}

impl Default for CompleteJpegData {
    fn default() -> Self {
        Self {
            width: Default::default(),
            height: Default::default(),
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

/// 量化表也是 Zigzag 形式存储的！！！
fn parse_dqt(block: &[u8]) -> io::Result<QuantizationTable> {
    let mut buf = ByteBuffer::from_bytes(block);
    let mut ret = QuantizationTable(Default::default());
    let precision_and_id = buf.read_u8()?;
    let _id = precision_and_id & 0x0F; // 忽略 ID，假设按顺序。
    let precision = precision_and_id >> 4;

    let output = &mut ret.0;

    let mut x = 0;
    let mut y = 0;
    let mut idx = 0;

    while idx < 64 {
        while idx < 64 {
            output[x][y] = if precision == 0 {
                buf.read_u8()? as u16
            } else {
                buf.read_u16()? as u16
            };
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
        while idx < 64 {
            output[x][y] = if precision == 0 {
                buf.read_u8()? as u16
            } else {
                buf.read_u16()? as u16
            };
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

    Ok(ret)
}

fn parse_sof0(block: &[u8], jpeg_data: &mut CompleteJpegData) -> io::Result<Vec<TempComponent>> {
    let mut buf = ByteBuffer::from_bytes(block);
    let mut ret = vec![];

    let precision = buf.read_u8()?; // 忽略精度，假设总是 8。
    if precision != 8 {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Unsupported precision",
        ));
    }
    jpeg_data.height = buf.read_u16()? as usize;
    jpeg_data.width = buf.read_u16()? as usize;
    let n_components = buf.read_u8()?;
    if n_components != 3 {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Unsupported number of components",
        ));
    }
    for _ in 0..n_components {
        let _id = buf.read_u8()?; // 忽略 ID，假设按顺序。
        let sampling_factors = buf.read_u8()?;
        let horizontal_sampling_factor = sampling_factors >> 4;
        let vertical_sampling_factor = sampling_factors & 0x0F;
        let quatization_table_id = buf.read_u8()?;
        ret.push(TempComponent {
            horizontal_sampling_factor,
            vertical_sampling_factor,
            quatization_table_id,
            dc_huffman_table_id: Default::default(),
            ac_huffman_table_id: Default::default(),
        });
    }

    Ok(ret)
}

/// 返回 (霍夫曼表, 类别, ID)。
fn parse_dht(block: &[u8]) -> io::Result<(CachedHuffmanTable, u8, u8)> {
    let mut buf = ByteBuffer::from_bytes(block);
    let mut ret = JpegHuffmanTable::new();

    let table_class_and_id = buf.read_u8()?;
    let table_class = table_class_and_id >> 4;
    let id = table_class_and_id & 0x0F;

    for i in 0..ret.codes.len() {
        ret.codes[i] = buf.read_u8()?;
    }
    while buf.get_rpos() < buf.len() {
        let value = buf.read_u8()?;
        ret.values.push(value);
    }

    Ok((ret.to_cached(), table_class, id))
}

fn parse_sos(block: &[u8], temp_components: &mut Vec<TempComponent>) -> io::Result<()> {
    let mut buf = ByteBuffer::from_bytes(block);

    let n_components = buf.read_u8()? as usize;
    for i in 0..n_components {
        let _id = buf.read_u8()?;
        let huffman_tables = buf.read_u8()?;
        let dc_huffman_table_id = huffman_tables >> 4;
        let ac_huffman_table_id = huffman_tables & 0x0F;
        temp_components[i].dc_huffman_table_id = dc_huffman_table_id;
        temp_components[i].ac_huffman_table_id = ac_huffman_table_id;
    }

    Ok(())
}

fn parse_image_data(buf: &mut ByteBuffer) -> io::Result<BitVec> {
    let mut ret = BitVec::new();

    let mut is_pre_ff = false;
    while buf.get_rpos() < buf.len() {
        let byte = buf.read_u8()?;

        if !is_pre_ff && byte != 0xFF || is_pre_ff && byte == 0x00 {
            let byte = if is_pre_ff { 0xFF } else { byte };
            let mut bits = byte.view_bits::<Lsb0>().to_owned();
            bits.reverse();
            ret.append(&mut bits);
        } else if !is_pre_ff && byte == 0xFF {
            // Skip.
        } else if is_pre_ff && byte == 0xD9 {
            // EOI.
            break;
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid image data",
            ));
        }

        is_pre_ff = if byte == 0xFF { true } else { false };
    }

    Ok(ret)
}

/// 第一步：从原始的 JPEG 数据中解析出解码所需的完整数据。
pub fn decode_step1(buf: &[u8]) -> io::Result<CompleteJpegData> {
    let mut ret = CompleteJpegData::default();
    let mut temp_components = vec![]; // 忽略 ID，假设分量按顺序。
    let mut quantization_tables = vec![];
    let mut huffman_tables = BTreeMap::<(u8, u8), Rc<CachedHuffmanTable>>::new();

    let mut buf = ByteBuffer::from_bytes(buf);
    buf.set_endian(Endian::BigEndian);
    while buf.get_rpos() < buf.len() {
        buf.read_u8().and_then(|v| {
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
                let block = read_block(&mut buf)?;
                let _app0 = parse_app0(&block)?; // 不使用。
            }
            // APPn
            0xE1..=0xEF => {
                let _block = read_block(&mut buf)?;
            }
            // DQT
            0xDB => {
                let block = read_block(&mut buf)?;
                let dqt = parse_dqt(&block)?;
                quantization_tables.push(Rc::new(dqt));
            }
            // SOF0（不支持 SOF2）
            0xC0 => {
                let block = read_block(&mut buf)?;
                temp_components = parse_sof0(&block, &mut ret)?;
            }
            // DHT
            0xC4 => {
                let block = read_block(&mut buf)?;
                let (table, table_class, id) = parse_dht(&block)?;
                huffman_tables.insert((table_class, id), Rc::new(table));
            }
            // SOS and image data
            0xDA => {
                let block = read_block(&mut buf)?;
                parse_sos(&block, &mut temp_components)?;
                ret.scan = parse_image_data(&mut buf)?;
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid block type 0x{:02X}", block_type),
                ));
            }
        }
    }

    for t in temp_components {
        let component = Component {
            horizontal_sampling_factor: t.horizontal_sampling_factor,
            vertical_sampling_factor: t.vertical_sampling_factor,
            quatization_table: quantization_tables[t.quatization_table_id as usize].clone(),
            dc_huffman_table: huffman_tables[&(0, t.dc_huffman_table_id)].clone(),
            ac_huffman_table: huffman_tables[&(1, t.ac_huffman_table_id)].clone(),
        };
        ret.components.push(component);
    }

    Ok(ret)
}

fn read_block(buf: &mut ByteBuffer) -> Result<Vec<u8>, io::Error> {
    let length = buf.read_u16()?;
    if length < 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid block length",
        ));
    }
    let length = length as usize - 2;
    let block = buf.read_bytes(length)?;
    Ok(block)
}
