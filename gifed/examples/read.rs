use std::fs::File;

use gifed::{
	reader::{self, GifReader},
	writer::ImageBuilder,
	Gif,
};

fn main() {
	let reader = GifReader::file("examples/simulation.gif").unwrap();

	// Create the directory we're we'll dump all the PNGs
	std::fs::create_dir_all("examples/read/").unwrap();

	for (frame_number, image) in reader.images().enumerate() {
		let filename = format!("examples/read/simulation_{}.png", frame_number);
		let file = File::create(filename).unwrap();

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
