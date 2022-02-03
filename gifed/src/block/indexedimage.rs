use std::convert::TryFrom;

use super::{ColorTable, ImageDescriptor};
use crate::LZW;

pub struct IndexedImage {
	pub image_descriptor: ImageDescriptor,
	pub local_color_table: Option<ColorTable>,
	pub indicies: Vec<u8>,
}

impl IndexedImage {
	pub fn left(&self) -> u16 {
		self.image_descriptor.left
	}

	pub fn top(&self) -> u16 {
		self.image_descriptor.left
	}

	pub fn width(&self) -> u16 {
		self.image_descriptor.width
	}

	pub fn height(&self) -> u16 {
		self.image_descriptor.height
	}

	pub fn as_boxed_slice(&self, minimum_code_size: u8) -> Box<[u8]> {
		let mut out = vec![];

		let mut boxed: Box<[u8]> = (&self.image_descriptor).into();
		out.extend_from_slice(&*boxed);

		// Get the mcs while we write out the color table
		let mut mcs = if let Some(lct) = &self.local_color_table {
			boxed = lct.into();
			out.extend_from_slice(&*boxed);

			lct.packed_len() + 1
		} else {
			minimum_code_size + 1
		};

		if mcs < 2 {
			mcs = 2; // Must be true: 0 <= mcs <= 8
		}

		// First write out the MCS
		out.push(mcs);

		let compressed = LZW::encode(mcs, &self.indicies);

		for chunk in compressed.chunks(255) {
			out.push(chunk.len() as u8);
			out.extend_from_slice(chunk);
		}
		// Data block length 0 to indicate an end
		out.push(0x00);

		out.into_boxed_slice()
	}
}

pub struct CompressedImage {
	pub image_descriptor: ImageDescriptor,
	pub local_color_table: Option<ColorTable>,
	pub lzw_minimum_code_size: u8,
	pub blocks: Vec<Vec<u8>>,
}
