use std::{convert::TryInto, fs::File, io::Write, path::Path};

use gifed::{
	block::{extension::GraphicControl, Block, ColorTable, IndexedImage, Version},
	reader::GifReader,
	writer::{GifBuilder, ImageBuilder},
	Color,
};
use owo_colors::OwoColorize;

fn main() {
	let file = if let Some(file) = std::env::args().skip(1).next() {
		file
	} else {
		println!("usage: gifprobe file.gif");
		return;
	};

	let gif = GifReader::file(&file).unwrap();

	println!("Version {}", gif.header.yellow());
	println!(
		"Logical Screen Descriptor\n\tDimensions {}x{}",
		gif.screen_descriptor.width.yellow(),
		gif.screen_descriptor.height.yellow()
	);

	if gif.screen_descriptor.color_table_present() {
		println!(
			"\tGlobal Color Table Present {}\n\tGlobal Color Table Size {}",
			"Yes".green(),
			gif.screen_descriptor.color_table_len().green()
		);
	} else {
		println!(
			"\tGlobal Color Table Present {}\n\tGlobal Color Table Size {}",
			"No".red(),
			gif.screen_descriptor.color_table_len().red()
		);
	}

	let mut img_count = 0;
	let mut hundreths: usize = 0;

	for block in gif.blocks {
		match block {
			Block::IndexedImage(img) => {
				describe_image(&img);
				img_count += 1;
			}
			Block::GraphicControlExtension(gce) => {
				hundreths += gce.delay_time() as usize;

				println!(
					"Graphic Control Extension\n\tDelay Time {}\n\tDispose {}",
					format!("{}s", gce.delay_time() as f32 / 100.0).yellow(),
					gce.disposal_method().unwrap().yellow()
				)
			}
			Block::LoopingExtension(_) => todo!(),
			Block::CommentExtension(cmt) => {
				println!("Comment Extension\n\tLength {}", cmt.len())
			}
			Block::ApplicationExtension(app) => {
				let auth = app.authentication_code();
				println!(
					"Application Extension\n\tIdentifier {}\n\tAuthentication {:02X} {:02X} {:02X}",
					app.identifier().yellow(),
					auth[0].yellow(),
					auth[1].yellow(),
					auth[2].yellow()
				);
			}
		}
	}

	println!(
		"{} is {}.{}s long and has {} frames",
		file,
		hundreths / 100,
		hundreths % 100,
		img_count
	);
}

fn describe_image(bli: &IndexedImage) {
	println!(
		"Image\n\tOffset {}x{}\n\tDimensions {}x{}",
		bli.image_descriptor.left.yellow(),
		bli.image_descriptor.top.yellow(),
		bli.image_descriptor.width.yellow(),
		bli.image_descriptor.height.yellow(),
	);

	if bli.image_descriptor.color_table_present() {
		println!(
			"\tLocal Color Table Present {}\n\tLocal Color Table Size {}",
			"Yes".green(),
			bli.image_descriptor.color_table_size().green()
		);
	} else {
		println!(
			"\tLocal Color Table Present {}\n\tLocal Color Table Size {}",
			"No".red(),
			bli.image_descriptor.color_table_size().red()
		);
	}
}
