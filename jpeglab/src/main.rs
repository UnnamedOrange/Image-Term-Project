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

fn main() {
    let args = Args::parse();
    let path = Path::new(&args.input);
}
