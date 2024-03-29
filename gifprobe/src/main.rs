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
	let file = if let Some(file) = std::env::args().nth(1) {
		file
	} else {
		println!("usage: gifprobe file.gif");
		return;
	};

	let mut expand = false;
	let mut colors = false;
	let args: Vec<String> = std::env::args().skip(2).collect();
	for cmd in args {
		match cmd.as_str() {
			"expand" => expand = true,
			"colors" | "colours" => colors = true,
			_ => {
				eprintln!("{cmd} is not a valid subcommand");
				return;
			}
		}
	}

	let decoder = Decoder::file(&file).unwrap();
	let mut reader = decoder.read().unwrap();

	println!("Version {}", reader.version.yellow());
	println!(
		"Logical Screen Descriptor\n\tDimensions {}x{}",
		reader.screen_descriptor.width.yellow(),
		reader.screen_descriptor.height.yellow()
	);

	if let Some(plt) = reader.palette.as_ref() {
		println!(
			"\tGlobal Color Table Present {}\n\tGlobal Color Table Size {}",
			"Yes".green(),
			reader.screen_descriptor.color_table_len().green()
		);

		if colors {
			for (idx, clr) in plt.iter().enumerate() {
				println!(
					"\t{} {}, {}, {}",
					idx.color(owo_colors::Rgb(clr.r, clr.g, clr.b)),
					clr.r,
					clr.g,
					clr.b
				);
			}
		}
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
				describe_image(img, offset, expand, colors);
				img_count += 1;
			}
			Block::GraphicControlExtension(gce) => {
				hundreths += gce.delay() as usize;

				let dispose_string = if let Some(dispose) = gce.disposal_method() {
					dispose.to_string()
				} else {
					format!(
						"Reserved: {:b} [packed {:08b}]",
						gce.packed().disposal_method(),
						gce.packed().raw
					)
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
					let looping = u16::from_le_bytes([data[1], data[2]]);

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

fn describe_image(bli: CompressedImage, offset: Range<usize>, expand: bool, colors: bool) {
	print!("Image");
	print_offset(offset);

	println!(
		"\tOffset {}x{}\n\tDimensions {}x{}",
		bli.image_descriptor.left.yellow(),
		bli.image_descriptor.top.yellow(),
		bli.image_descriptor.width.yellow(),
		bli.image_descriptor.height.yellow(),
	);

	if expand {
		println!("\tLZW Code Size {}", bli.lzw_code_size.yellow());
	}

	if let Some(plt) = bli.palette().as_ref() {
		println!(
			"\tLocal Color Table Present {}\n\tLocal Color Table Size {}",
			"Yes".green(),
			bli.image_descriptor.color_table_size().green()
		);

		if colors {
			for (idx, clr) in plt.iter().enumerate() {
				println!("\t{idx} {}, {}, {}", clr.r, clr.g, clr.b);
			}
		}
	} else {
		println!(
			"\tLocal Color Table Present {}\n\tLocal Color Table Size {}",
			"No".red(),
			bli.image_descriptor.color_table_size().red()
		);
	}

	match bli.decompress() {
		Err(e) => {
			println!("\tDecompress Failed {}", e.to_string().red());
		}
		Ok(img) if colors => {
			let mut indicie_count = vec![0; 256];
			let mut unique = 0;
			for idx in img.indicies {
				if indicie_count[idx as usize] == 0 {
					unique += 1;
				}

				indicie_count[idx as usize] += 1;
			}

			println!("\tUnique Indicies {}", unique.yellow());

			for (idx, &count) in indicie_count.iter().enumerate() {
				if count > 0 {
					println!("\t\t{idx} {count}");
				}
			}
		}
		Ok(_) => (),
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
