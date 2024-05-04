use std::io;
use std::path::Path;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(
        help = "Input image file",
        long_help = "Input image file. To compress an image, the extension must be bmp. To uncompress an image, the extension must be jpg."
    )]
    input: String,
}

fn handle_others(path: &Path) -> io::Result<()> {
    todo!()
}

fn handle_jpg(path: &Path) -> io::Result<()> {
    todo!()
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let path = Path::new(&args.input);
    match path.extension().and_then(|v| v.to_str()) {
        Some("jpg") => {
            println!(
                "[INFO] 输入 JPEG 文件 {}，解压为位图",
                path.to_str().unwrap_or_default()
            );
            handle_jpg(path)
        }
        _ => {
            println!(
                "[INFO] 输入其他格式的图片文件 {}，压缩为 JPEG",
                path.to_str().unwrap_or_default()
            );
            handle_others(path)
        }
    }
}
