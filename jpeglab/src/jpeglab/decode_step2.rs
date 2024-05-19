use std::io;

use bitvec::field::BitField;
use bitvec::vec::BitVec;

use super::decode_step1::CompleteJpegData;
use super::decode_step1::Component;
use super::encode_step5::ZigzagDu;
use super::encode_step6::CachedHuffmanTable;

#[derive(Debug)]
pub struct DecodeZigzagMcuCollection {
    pub width: usize,
    pub height: usize,
    pub components: Vec<Component>,
    pub zigzag_dus: Vec<ZigzagDu>,
}

struct DcDecoder<'a> {
    pub sum: i16,
    pub huffman_table: &'a CachedHuffmanTable,
}

struct AcDecoder<'a> {
    pub huffman_table: &'a CachedHuffmanTable,
}

fn entropy_decode_category(
    scan: &BitVec,
    offset: &mut usize,
    huffman_table: &CachedHuffmanTable,
) -> io::Result<u8> {
    let ht = &huffman_table.0;
    let mut symbol = None;
    for (k, v) in ht {
        if scan[*offset..].starts_with(&v) {
            symbol = Some(*k);
            *offset += v.len();
            break;
        }
    }
    if let Some(symbol) = symbol {
        Ok(symbol)
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Fail to decode DC coefficient",
        ))
    }
}

fn entropy_decode_value(scan: &BitVec, offset: &mut usize, category: u8) -> io::Result<i16> {
    if category >> 4 != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid category for a value",
        ));
    }

    if category == 0 {
        return Ok(0);
    }
    let mut bits = scan[*offset..*offset + category as usize].to_owned();
    *offset += category as usize;
    let is_positive = bits[0];
    bits.reverse(); // MSB0 to LSB0.
    let abs_bits = if is_positive {
        bits
    } else {
        bits.iter().map(|v| !v).collect()
    };
    let abs_value = abs_bits.load::<u16>();
    let value = if is_positive {
        abs_value as i16
    } else {
        -(abs_value as i16)
    };
    Ok(value)
}

impl<'a> DcDecoder<'a> {
    fn new(huffman_table: &'a CachedHuffmanTable) -> Self {
        Self {
            sum: 0,
            huffman_table,
        }
    }

    fn decode(&mut self, scan: &BitVec, offset: &mut usize) -> io::Result<i16> {
        let category = entropy_decode_category(scan, offset, &self.huffman_table)?;
        let diff = entropy_decode_value(scan, offset, category)?;
        self.sum += diff;
        Ok(self.sum)
    }
}

impl<'a> AcDecoder<'a> {
    fn new(huffman_table: &'a CachedHuffmanTable) -> Self {
        Self { huffman_table }
    }

    fn decode(&self, scan: &BitVec, offset: &mut usize, du: &mut [i16; 64]) -> io::Result<()> {
        let mut idx = 1;
        while idx < du.len() {
            let symbol = entropy_decode_category(scan, offset, &self.huffman_table)?;
            if symbol == 0x00 {
                // EOB
                while idx < du.len() {
                    du[idx] = 0;
                    idx += 1;
                }
                break;
            }

            let zrl = symbol >> 4;
            for _ in 0..zrl {
                if idx >= du.len() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "AC coefficients exceeded",
                    ));
                }
                du[idx] = 0;
                idx += 1;
            }
            let category = symbol & 0x0F;
            if idx >= du.len() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "AC coefficients exceeded",
                ));
            }
            du[idx] = entropy_decode_value(scan, offset, category)?;
            idx += 1;
        }
        Ok(())
    }
}

/// 第二步：解码熵编码，得到一系列 Zigzag 形式的 DU。
pub fn decode_step2(jpeg_data: &CompleteJpegData) -> io::Result<DecodeZigzagMcuCollection> {
    let mut zigzag_dus = vec![];

    let scan = &jpeg_data.scan;
    let mut dc_decoders = Vec::<DcDecoder>::new();
    for component in &jpeg_data.components {
        dc_decoders.push(DcDecoder::new(&component.dc_huffman_table));
    }

    let mut offset = 0;
    while offset < scan.len() {
        // MCU。
        for (i, component) in jpeg_data.components.iter().enumerate() {
            // 一个分量连续存储 H * V 个 DU。
            let sf = component.horizontal_sampling_factor * component.vertical_sampling_factor;
            for _ in 0..sf {
                let mut du = [0; 64];

                // DC 系数。
                du[0] = dc_decoders[i].decode(scan, &mut offset)?;

                // AC 系数。
                let ac_decoder = AcDecoder::new(&component.ac_huffman_table);
                ac_decoder.decode(scan, &mut offset, &mut du)?;

                zigzag_dus.push(ZigzagDu(du));
            }
        }
    }

    Ok(DecodeZigzagMcuCollection {
        width: jpeg_data.width,
        height: jpeg_data.height,
        components: jpeg_data.components.clone(),
        zigzag_dus,
    })
}
