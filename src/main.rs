use std::io::{BufRead, BufReader};

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// File path of the CSV file
    #[clap(short, long)]
    csv_path: String,
    /// Output file path
    #[clap(short, long)]
    out_path: String,
    /// Input file path (White if none)
    #[clap(short, long)]
    input_path: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let csv_file = std::fs::OpenOptions::new().read(true).open(args.csv_path)?;

    // Ensure out-file can be created
    drop(
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&args.out_path)?,
    );
    // TODO: input_path
    let mut image = vec![255u8; 2000 * 2000 * 4];

    let mut reader = BufReader::new(csv_file);
    let mut buffer = String::new();
    let mut c = 0;

    // Skip header
    reader.read_line(&mut buffer);
    buffer.clear();

    while let Ok(_) = reader.read_line(&mut buffer) {
        c += 1;
        let line = buffer.trim();
        if line.is_empty() {
            break;
        }

        let color_and_pos = line.split_once(',').unwrap().1.split_once(',').unwrap().1;
        let (color, pos_str) = color_and_pos.split_once(',').unwrap();

        let pos_str = pos_str.trim_matches('"');
        let (x_str, y_str) = pos_str.split_once(',').unwrap();
        let (x, y) = (x_str.parse::<u16>()?, y_str.parse::<u16>()?);

        let color = color.trim_start_matches('#').as_bytes();
        let (r, g, b) = (&color[..2], &color[2..4], &color[4..]);
        let (r, g, b) = (
            btoi::btoi_radix::<u8>(r, 16)?,
            btoi::btoi_radix::<u8>(g, 16)?,
            btoi::btoi_radix::<u8>(b, 16)?,
        );
        if c % 5000 == 0 {
            println!("{}: x:{} y:{} col: {} {} {}", c, x, y, r, g, b);
        }
        let idx = (x as usize) + (y as usize) * 2000;
        image[idx * 4 + 0] = r;
        image[idx * 4 + 1] = g;
        image[idx * 4 + 2] = b;
        //image[idx * 4 + 3] = 255;
        /*
        image[(x as usize) + (y as usize) * 2000] =
            (r as u32) & ((g as u32) << 8) & ((b as u32) << 16) & (255u32 << 24);
        */

        buffer.clear();
    }

    image::RgbaImage::from_vec(2000, 2000, image)
        .unwrap()
        .save_with_format(args.out_path, image::ImageFormat::Png);
    Ok(())
}
