use std::io::{BufRead, BufReader};

use clap::Parser;
use image::GenericImageView;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// File path of the CSV file
    #[clap(short, long)]
    csv_path: String,
    /// Output image file path
    #[clap(short, long)]
    out_path: String,
    /// Input image file path (White if none)
    #[clap(short, long)]
    in_path: Option<String>,
    /// Number of pixel placements to apply. 0=all
    #[clap(short, long, default_value_t = 0)]
    lines_to_do: usize,
    /// Number of pixel placements to skip
    #[clap(short, long, default_value_t = 0)]
    lines_to_skip: usize,
    /// Find continuous runs of white pixels instead of making an image.
    /// This prints all runs >=100 pixels and they line number
    #[clap(short, long)]
    runs_of_white: bool,
    /// Filter lines to just admin rectangles
    #[clap(short, long)]
    find_admin_rects: bool,
}
// Example usage:
//  cargo run --release -- --csv-path .\data\2022_rplace_cleaned.csv --out-path out.png --in-path .\150000000.png --lines-to-do 6028080 --lines-to-skip 150000000
//  cargo run --release -- --csv-path .\data\2022_rplace_cleaned.csv --runs-of-white --out-path none

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let csv_file = std::fs::OpenOptions::new().read(true).open(args.csv_path)?;
    let mut reader = BufReader::new(csv_file);
    let mut buffer = String::new();
    let mut c = 0;

    // Skip header
    reader.read_line(&mut buffer).unwrap();
    buffer.clear();

    if args.runs_of_white {
        let mut last_non_white = 0;
        let mut was_white = false;
        while let Ok(_) = reader.read_line(&mut buffer) {
            c += 1;
            let line = buffer.trim();
            if line.is_empty() {
                break;
            }
            if c % 10000000 == 0 {
                println!("Progress:{}", last_non_white);
            }

            was_white = buffer.contains(",#FFFFFF,");
            if !was_white {
                let run_length = c - last_non_white;
                if run_length > 100 {
                    println!(
                        "Found {} lines long run of white at line {}",
                        run_length,
                        last_non_white + 1
                    );
                }
                last_non_white = c;
            }

            buffer.clear();
        }
        let final_run_length = c - last_non_white;
        if final_run_length > 100 && was_white {
            println!(
                "Found {} lines long run of white at line {}",
                final_run_length,
                last_non_white + 1
            );
        }

        return Ok(());
    }

    // Ensure out-file can be created
    drop(
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&args.out_path)?,
    );
    let mut image = vec![255u8; 2000 * 2000 * 4];
    if let Some(in_path) = args.in_path.as_ref() {
        let input_file = std::fs::OpenOptions::new().read(true).open(in_path)?;
        let reader = BufReader::new(input_file);

        let img = image::load(reader, image::ImageFormat::Png).unwrap();
        assert!(img.width() == 2000 && img.height() == 2000);
        for x in 0..2000 {
            for y in 0..2000 {
                let idx = (x as usize) + (y as usize) * 2000;
                let px: image::Rgba<u8> = img.get_pixel(x, y);
                image[idx * 4 + 0] = px.0[0];
                image[idx * 4 + 1] = px.0[1];
                image[idx * 4 + 2] = px.0[2];
            }
        }
    }

    c = 0;
    if args.lines_to_skip != 0 {
        while let Ok(_) = reader.read_line(&mut buffer) {
            c += 1;
            if c == args.lines_to_skip {
                break;
            }
            buffer.clear();
        }
        buffer.clear();
    }
    while let Ok(_) = reader.read_line(&mut buffer) {
        c += 1;
        let line = buffer.trim();
        if line.is_empty() {
            break;
        }

        let color_and_pos = line.split_once(',').unwrap().1.split_once(',').unwrap().1;
        let (color, pos_str) = color_and_pos.split_once(',').unwrap();

        let (x_str, y_str) = pos_str.split_once(',').unwrap();
        let (x, y) = (x_str.parse::<u16>()?, y_str.parse::<u16>()?);

        let color = color.trim_start_matches('#').as_bytes();
        let (r, g, b) = (&color[..2], &color[2..4], &color[4..]);
        let (r, g, b) = (
            btoi::btoi_radix::<u8>(r, 16)?,
            btoi::btoi_radix::<u8>(g, 16)?,
            btoi::btoi_radix::<u8>(b, 16)?,
        );
        //total: 160350000
        if c % 1000000 == 0 {
            println!(
                "Progress line {}: x:{} y:{} col: {} {} {}",
                c, x, y, r, g, b
            );
        }
        let idx = (x as usize) + (y as usize) * 2000;
        image[idx * 4 + 0] = r;
        image[idx * 4 + 1] = g;
        image[idx * 4 + 2] = b;
        /* reddit parser version:
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
        */

        buffer.clear();
        if c == args.lines_to_do + args.lines_to_skip {
            break;
        }
    }

    image::RgbaImage::from_vec(2000, 2000, image)
        .unwrap()
        .save_with_format(args.out_path, image::ImageFormat::Png)
        .unwrap();
    Ok(())
}
