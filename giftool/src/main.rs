use std::{fs::File, path::PathBuf};

use gifed::reader::GifReader;

fn main() {
	let subcommand = std::env::args().nth(1);

	match subcommand.as_ref().map(|s| s.as_str()) {
		Some("extract-frames") => extract_frames(),
		Some("analyze") => analyze(),
		_ => print_usage_and_exit(),
	}
}

fn print_usage_and_exit() -> ! {
	println!("usage: giftool <subcommand> <options>\n");
	println!("extract_frames <input_gif> <output_directory>");
	println!("\tExtract each frame of the gif to a png in the output directory.");

	std::process::exit(0)
}

fn extract_frames() {
	let input_gif = std::env::args().nth(2).map(PathBuf::from);
	let out_dir = std::env::args().nth(3).map(PathBuf::from);

	let input_gif = if let Some(igif) = input_gif {
		if !igif.exists() {
			println!("The path provided to the gif does not exist");
			std::process::exit(1);
		} else if !igif.is_file() {
			println!("The path provided to the gif is not a file");
			std::process::exit(1);
		}

		igif
	} else {
		println!("No gif file provided");
		std::process::exit(1);
	};

	let out_dir = if let Some(odir) = out_dir {
		if !odir.exists() {
			println!("The output path does not exist");
			std::process::exit(1);
		} else if !odir.is_dir() {
			println!("The output path is not a directory");
			std::process::exit(1);
		}

		odir
	} else {
		println!("No output directory provided");
		std::process::exit(1);
	};

	let gread = match GifReader::file(&input_gif) {
		Ok(gread) => gread,
		Err(e) => {
			println!(
				"Failed to read {} as a gif:\n{}",
				input_gif.to_string_lossy(),
				e
			);
			std::process::exit(1);
		}
	};

	for (frame_number, image) in gread.images().enumerate() {
		let mut fname = out_dir.clone();
		fname.push(format!("{}.png", frame_number));

		let file = File::create(fname).unwrap();

		let mut encoder = png::Encoder::new(file, image.width as u32, image.height as u32);
		encoder.set_color(png::ColorType::Indexed);
		encoder.set_palette(image.palette.as_bytes());

		if let Some(trns) = image.png_trns() {
			encoder.set_trns(trns);
		}

		let mut writer = encoder.write_header().unwrap();
		writer.write_image_data(&image.indicies).unwrap();
	}
}

fn analyze() {}
