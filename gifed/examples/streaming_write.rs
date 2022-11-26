use std::fs::File;

use gifed::{
	block::{LoopCount, Palette},
	writer::{ImageBuilder, Writer},
	Color, EncodeError, Gif,
};

fn main() -> Result<(), EncodeError> {
	let gif_path = match std::env::args().nth(1) {
		None => {
			eprintln!("Expected a path to output the gif to");
			std::process::exit(-1);
		}
		Some(path) => path,
	};

	let mut palette = Palette::new();

	// Fill the palette with every gray
	for gray in 0..=255 {
		palette.push(Color::new(gray, gray, gray));
	}

	let mut image = vec![0; 128 * 128];

	// Create a file to write the gif to. We can try here, with the ?, because
	// EncodeError has a From<std::io::Error> impl
	let file = File::create(gif_path)?;
	let mut writer = Writer::new(file, 128, 128, Some(palette))?;
	for idx in 0..=255 {
		image.fill(idx);

		writer.image(ImageBuilder::new(128, 128).delay(3).build(image.clone())?)?;
	}

	writer.repeat(LoopCount::Forever)?;
	writer.done()
}
