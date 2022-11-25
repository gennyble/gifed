use gifed::{
	block::{LoopCount, Palette},
	writer::ImageBuilder,
	Color, Gif,
};

fn main() {
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

	let mut builder = Gif::builder(128, 128).palette(palette);
	for idx in 0..=255 {
		image.fill(idx);

		builder = builder.image(
			ImageBuilder::new(128, 128)
				.delay(3)
				.build(image.clone())
				.unwrap(),
		);
	}

	builder
		.repeat(LoopCount::Forever)
		.build()
		.unwrap()
		.save(gif_path)
		.unwrap();
}
