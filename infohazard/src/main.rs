use std::fs::File;

use gifed::{
	block::{LoopCount, Palette},
	writer::{GifBuilder, ImageBuilder, Writer},
};

fn main() {
	let black = vec![0; 35];
	let red = vec![1; 35];

	let mut horiz = black.clone();
	for _ in 0..(34 / 2) {
		horiz.extend(&red);
		horiz.extend(&black);
	}

	let mut row = vec![0];
	row.extend([2, 0].repeat(34 / 2));

	let vert = row.repeat(35);

	let file = File::create("infohazard.gif").unwrap();
	let mut writer = Writer::new(
		file,
		35,
		35,
		Some(Palette::try_from([0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 0].as_slice()).unwrap()),
	)
	.unwrap();

	writer.repeat(LoopCount::Forever).unwrap();
	writer
		.image(ImageBuilder::new(35, 35).delay(16).build(horiz).unwrap())
		.unwrap();
	writer
		.image(ImageBuilder::new(35, 35).delay(16).build(vert).unwrap())
		.unwrap();
	writer.done().unwrap();

	/*
	GifBuilder::new(35, 35)
		.palette(Palette::try_from([0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 0].as_slice()).unwrap())
		.repeat(LoopCount::Forever)
		.image(ImageBuilder::new(35, 35).delay(16).build(horiz).unwrap())
		.image(ImageBuilder::new(35, 35).delay(16).build(vert).unwrap())
		.build()
		.unwrap()
		.save("infohazard.gif")
		.unwrap()*/
}
