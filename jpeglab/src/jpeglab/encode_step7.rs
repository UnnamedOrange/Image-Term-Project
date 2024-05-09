use std::io;
use std::path::Path;

use super::encode_step6::JpegOutputData;

/// 第七步：输出 JPEG 文件。
/// 文件名为 out.jpg。
pub fn encode_step7(data: &JpegOutputData) -> io::Result<()> {
    let out_path = Path::new("out.jpg");

    todo!()
}
