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

/// 第七步：输出 JPEG 文件。
/// 文件名为 out.jpg。
pub fn encode_step7(data: &JpegOutputData) -> io::Result<()> {
    let out_path = Path::new("out.jpg");

    todo!()
}
