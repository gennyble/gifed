use std::{fs::File, ops::Deref, path::PathBuf};

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
	println!("analyze <input_gif>");
	println!("\tAnalyze the gif, looking for places to make the image smaller.");

	std::process::exit(0)
}

fn check_input_gif(igif: Option<PathBuf>) -> PathBuf {
	if let Some(igif) = igif {
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
	}
}

fn extract_frames() {
	let input_gif = std::env::args().nth(2).map(PathBuf::from);
	let out_dir = std::env::args().nth(3).map(PathBuf::from);

	let input_gif = check_input_gif(input_gif);

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

fn analyze() {
	let input_gif = check_input_gif(std::env::args().nth(2).map(PathBuf::from));

	//TODO:
	// Look at color tables:
	// - Can any of them be combined into a global colour table? Can they all be?
	// - Can the colour tables be shrank? Is it bigger than it needs to be. (Are all the inidicies used)

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

	if gread.global_color_table.is_some() {
		if analyze_can_be_a_gct(&gread) {
			println!("All local colors are in the global, this gif doesn't need locals");
		}
	}
}

/// A gif can be reindexed to not need any local color tables if:
/// - The global color table contains all colors used in the gif
/// OR
/// - The global color table does not contain all colors used in the gif,
///   but it can fit the colors it does not have.
/// OR
/// - There is no global color table, but there are only 256 colors used in
///   the gif, so it can be created.
///
/// # Returns
/// This function returns `true` if the gif can be reindexed to not need any
/// local tables anymore.
fn analyze_can_be_a_gct(gread: &Gif) -> bool {
	let mut gct = if let Some(gct) = gread.global_color_table.as_ref() {
		let mut gct = gct.clone().to_vec();
		gct.dedup();
		gct
	} else {
		vec![]
	};

	let mut gct_contains_all_colors = true;
	let mut largest_gct_seen = 0;
	let mut gif_has_local_tables = false;

	for image in gread.images() {
		if image.packed.color_table() {
			gif_has_local_tables = true;
			let lct = image.palette;

			for color in lct.deref() {
				if !gct.contains(color) {
					gct_contains_all_colors = false;
					gct.push(color.clone());
				}
			}
		} else {
			for index in image.indicies {
				largest_gct_seen = largest_gct_seen.max(*index);
			}
		}
	}

	if gct_contains_all_colors || !gif_has_local_tables {
		true
	} else if !gct_contains_all_colors && gct.len() <= 256 {
		true
	} else {
		false
	}
}
