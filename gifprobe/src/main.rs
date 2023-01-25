use std::ops::Range;

use gifed::{
	block::{
		Block::{self},
		CompressedImage,
	},
	reader::Decoder,
};
use owo_colors::OwoColorize;

fn main() {
	let file = if let Some(file) = std::env::args().skip(1).next() {
		file
	} else {
		println!("usage: gifprobe file.gif");
		return;
	};

	let expand = match std::env::args().nth(2).as_deref() {
		Some("expand") => true,
		None => false,
		Some(str) => {
			eprintln!("{str} is not recognised. Did you mean 'expand'?");
			return;
		}
	};

	let decoder = Decoder::file(&file).unwrap();
	let mut reader = decoder.read().unwrap();

	println!("Version {}", reader.version.yellow());
	println!(
		"Logical Screen Descriptor\n\tDimensions {}x{}",
		reader.screen_descriptor.width.yellow(),
		reader.screen_descriptor.height.yellow()
	);

	if reader.screen_descriptor.has_color_table() {
		println!(
			"\tGlobal Color Table Present {}\n\tGlobal Color Table Size {}",
			"Yes".green(),
			reader.screen_descriptor.color_table_len().green()
		);
	} else {
		println!(
			"\tGlobal Color Table Present {}\n\tGlobal Color Table Size {}",
			"No".red(),
			reader.screen_descriptor.color_table_len().red()
		);
	}

	let mut img_count = 0;
	let mut hundreths: usize = 0;

	loop {
		let block = match reader.block() {
			Ok(Some(block)) => block,
			Ok(None) => break,
			Err(e) => {
				eprintln!("error reading file: {e}");
				std::process::exit(-1);
			}
		};

		let offset = block.offset;
		let block = block.block;

		match block {
			Block::CompressedImage(img) => {
				describe_image(&img, offset);
				img_count += 1;
			}
			Block::GraphicControlExtension(gce) => {
				hundreths += gce.delay() as usize;

				let dispose_string = if let Some(dispose) = gce.disposal_method() {
					dispose.to_string()
				} else {
					format!("Reserved: {:b}", gce.packed().disposal_method())
				};

				print!("Graphic Control Extension");
				print_offset(offset);

				println!(
					"\tDelay Time {}\n\tDispose {}",
					format!("{}s", gce.delay() as f32 / 100.0).yellow(),
					dispose_string.yellow()
				)
			}
			Block::LoopingExtension(_) => todo!(),
			Block::CommentExtension(cmt) => {
				print!("Comment Extension");
				print_offset(offset);

				println!("\tLength {}", cmt.len().yellow());

				match String::from_utf8(cmt.clone()) {
					Ok(cmt) => println!("\tString \"{}\"", cmt.yellow()),
					Err(_) => {
						if !expand {
							println!("\tString {}", "Content is not utf8".red())
						} else {
							let lossy = String::from_utf8_lossy(&cmt);
							if lossy.len() > 0 {
								println!("\tString (lossy) \"{}\"", lossy.yellow())
							} else {
								println!(
									"\tString (lossy) \"{}\"",
									"Could not parse as UTF8 Lossy".red()
								);
							}
						}
					}
				}
			}
			Block::ApplicationExtension(app) => {
				let auth = app.authentication_code();
				let app_ident = String::from_utf8_lossy(app.identifier());

				print!("Application Extension");
				print_offset(offset);

				println!(
					"\tIdentifier {}\n\tAuthentication {:02X} {:02X} {:02X}",
					app_ident.yellow(),
					auth[0].yellow(),
					auth[1].yellow(),
					auth[2].yellow()
				);

				if app_ident == "NETSCAPE" {
					let data = app.data();
					let looping = u16::from_le_bytes([data[0], data[1]]);

					if looping == 0 {
						println!("\tLoop {}", "forever".yellow())
					} else {
						println!("\tLoop {}", looping.yellow());
					}
				} else {
					let data = app.data();

					match String::from_utf8(data.to_vec()) {
						Ok(s) => {
							println!(
								"\tData {}",
								format!("Valid UTF-8, {} bytes", s.len()).yellow()
							);

							if expand {
								println!("\tString \"{}\"", s.yellow());
							}
						}
						Err(_e) => println!(
							"\tData {}",
							format!("Invalid UTF-8, {} bytes", data.len()).yellow()
						),
					}
				}
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

fn describe_image(bli: &CompressedImage, offset: Range<usize>) {
	print!("Image");
	print_offset(offset);

	println!(
		"\tOffset {}x{}\n\tDimensions {}x{}",
		bli.image_descriptor.left.yellow(),
		bli.image_descriptor.top.yellow(),
		bli.image_descriptor.width.yellow(),
		bli.image_descriptor.height.yellow(),
	);

	if bli.image_descriptor.has_color_table() {
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

fn print_offset(offset: Range<usize>) {
	print!(" [");
	print_usize(offset.start);
	print!(" … ");
	print_usize(offset.end);
	println!("] ");
}

fn print_usize(offset: usize) {
	let bytes = offset.to_le_bytes();
	let mut seen_nonzero = false;
	for byte in bytes {
		if byte == 0 {
			if seen_nonzero {
				break;
			}
		} else {
			seen_nonzero = true;
		}

		print!("{:02X}", byte.cyan());
	}
}
