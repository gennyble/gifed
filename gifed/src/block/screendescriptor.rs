use std::convert::TryInto;

use super::{packed::ScreenPacked, ColorTable};

pub struct ScreenDescriptor {
	pub width: u16,
	pub height: u16,
	pub packed: ScreenPacked,
	pub background_color_index: u8,
	pub pixel_aspect_ratio: u8,
}

impl ScreenDescriptor {
	pub fn new(width: u16, height: u16) -> Self {
		Self {
			width,
			height,
			packed: ScreenPacked { raw: 0 },
			background_color_index: 0,
			pixel_aspect_ratio: 0,
		}
	}

	/// This data structure **does not** contain the color table, only a flag to
	/// indicate if one is present and it's size.
	pub fn set_color_table_metadata<T: AsRef<ColorTable>>(&mut self, table: Option<T>) {
		if let Some(table) = table {
			let table = table.as_ref();
			self.packed.set_color_table(true);
			self.packed.set_color_table_size(table.packed_len());
		} else {
			self.packed.set_color_table(false);
			// This is not strictly needed, but we'll clear it anyway
			self.packed.set_color_table_size(0);
		}
	}

	pub fn has_color_table(&self) -> bool {
		self.packed.color_table()
	}

	pub fn color_table_len(&self) -> usize {
		crate::packed_to_color_table_length(self.packed.color_table_size())
	}
}

impl From<&ScreenDescriptor> for Box<[u8]> {
	fn from(lsd: &ScreenDescriptor) -> Self {
		let mut vec = vec![];
		vec.extend_from_slice(&lsd.width.to_le_bytes());
		vec.extend_from_slice(&lsd.height.to_le_bytes());
		vec.push(lsd.packed.raw);
		vec.push(lsd.background_color_index);
		vec.push(lsd.pixel_aspect_ratio);

		vec.into_boxed_slice()
	}
}

impl From<[u8; 7]> for ScreenDescriptor {
	fn from(arr: [u8; 7]) -> Self {
		let width = u16::from_le_bytes(arr[0..2].try_into().unwrap());
		let height = u16::from_le_bytes(arr[2..4].try_into().unwrap());
		let packed = arr[4];
		let background_color_index = arr[5];
		let pixel_aspect_ratio = arr[6];

		Self {
			width,
			height,
			packed: ScreenPacked { raw: packed },
			background_color_index,
			pixel_aspect_ratio,
		}
	}
}
