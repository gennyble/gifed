use std::{ops::Deref, path::PathBuf};

use gifed::{reader::Decoder, Gif};

mod fix;

fn main() {
	let file = std::env::args().nth(1).unwrap();
	let arg = std::env::args().nth(2).map(|cmd| cmd.to_lowercase());

	let gif = Decoder::file(&file).unwrap().read_all().unwrap();

	let plt_report = same_palette(&gif);
	match plt_report {
		PaletteReport {
			has_local: true,
			local_redundant: true,
			local_matching_indicies,
		} => {
			if local_matching_indicies {
				println!("!!! LOCPLT_NORE. This could've been a global palette");
			} else {
				println!("!!  LOCPLT. This gif can be reindexed and have a global palette");
			}
		}
		PaletteReport {
			has_local: true,
			local_redundant: false,
			..
		} => {
			println!("    gif has local palettes and they differ");
		}
		PaletteReport {
			has_local: false, ..
		} => {
			println!("    gif only has a global palette");
		}
	}

	if arg.as_deref() == Some("fix") {
		if let Some(fix_gif) = fix::palette_errors(&gif, plt_report) {
			if !fix::images_match_exactly(&gif, &fix_gif) {
				panic!("fixed images did not exactly match, this is a hard error")
			}

			println!("--- fixing, writing!");
			let mut path = PathBuf::from(file);
			path.set_file_name(format!(
				"{}_fix",
				path.file_stem().unwrap().to_string_lossy()
			));

			fix_gif.save(path).unwrap();
		}
	}
}

pub struct PaletteReport {
	// Does the gif even contain local color tables?
	has_local: bool,
	// ... do those color tables always contain the same colors?
	local_redundant: bool,
	// ... and do those colors all have matching inidices, making it possible
	// to simply set the global palette and remove the locals?
	local_matching_indicies: bool,
}

fn same_palette(gif: &Gif) -> PaletteReport {
	let mut palette = gif.global_color_table.as_ref();
	let mut report = PaletteReport {
		has_local: false,
		local_redundant: true,
		local_matching_indicies: true,
	};

	for img in gif.images() {
		if let Some(local_palette) = img.compressed.palette() {
			report.has_local = true;

			match palette {
				None => palette = Some(local_palette),
				Some(known_palette) => {
					if !local_palette.eq(known_palette) {
						// Are the palletes equal, even?
						report.local_redundant = false;
						report.local_matching_indicies = false;
						return report;
					} else if report.local_matching_indicies {
						// it's matching, but are the indicies the same?
						for known_color in known_palette.deref() {
							for local_color in local_palette.deref() {
								if known_color != local_color {
									report.local_matching_indicies = false;
								}
							}
						}
					}
				}
			}
		}
	}

	report
}
