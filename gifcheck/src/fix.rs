use gifed::{
	block::{Block, Palette},
	Color, Gif,
};

use crate::PaletteReport;

pub fn palette_errors(gif: &Gif, report: PaletteReport) -> Option<Gif> {
	if report.local_matching_indicies {
		let mut new = gif.clone();

		for block in new.blocks.iter_mut() {
			if let Block::CompressedImage(comp) = block {
				comp.image_descriptor.packed.set_color_table(false);
				comp.image_descriptor.packed.set_color_table_size(0);

				if let Some(plt) = comp.local_color_table.take() {
					new.global_color_table.get_or_insert(plt);
				}
			}
		}

		Some(new)
	} else {
		None
	}
}

pub fn images_match_exactly(gifa: &Gif, gifb: &Gif) -> bool {
	let mut a_buf = vec![0; gifa.width() * gifa.height() * 4];
	let mut b_buf = vec![0; gifb.width() * gifb.height() * 4];

	for (a, b) in gifa.images().zip(gifb.images()) {
		if a.width() != b.width() || a.height() != b.height() {
			return false;
		}

		if a.left() != b.left() || a.top() != b.top() {
			return false;
		}

		let a_decomp = a.decompess();
		let b_decomp = b.decompess();

		let a_size = deindex(
			&a_decomp.indicies,
			a.palette(),
			a.transparent_index(),
			&mut a_buf,
		);

		let b_size = deindex(
			&b_decomp.indicies,
			b.palette(),
			b.transparent_index(),
			&mut b_buf,
		);

		match (a_size, b_size) {
			(None, _) | (_, None) => return false,
			(Some(asize), Some(bsize)) => {
				if asize != bsize {
					return false;
				}

				if a_buf[..asize] != b_buf[..bsize] {
					return false;
				}
			}
		}
	}

	true
}

fn deindex(indicies: &[u8], plt: &Palette, trns: Option<u8>, buffer: &mut [u8]) -> Option<usize> {
	let mut rgba = |idx: usize, clr: Option<Color>| match clr {
		None => {
			buffer[idx] = 0;
			buffer[idx + 1] = 0;
			buffer[idx + 2] = 0;
			buffer[idx + 3] = 0;
		}
		Some(clr) => {
			buffer[idx] = clr.r;
			buffer[idx + 1] = clr.g;
			buffer[idx + 2] = clr.b;
			buffer[idx + 3] = 255;
		}
	};

	for (idx, color_idx) in indicies.iter().enumerate() {
		match (trns, plt.get(*color_idx)) {
			(Some(trns_idx), _) if trns_idx == *color_idx => rgba(idx * 4, None),
			(_, Some(color)) => rgba(idx * 4, Some(color)),
			(Some(_) | None, None) => {
				return None;
			}
		}
	}

	Some(indicies.len() * 4)
}
