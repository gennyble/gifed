use std::convert::TryInto;

use super::{packed::ImagePacked, ColorTable};

pub struct ImageDescriptor {
	pub left: u16,
	pub top: u16,
	pub width: u16,
	pub height: u16,
	pub packed: ImagePacked,
}

impl ImageDescriptor {
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

	pub fn color_table_size(&self) -> usize {
		crate::packed_to_color_table_length(self.packed.color_table_size())
	}
}

impl From<&ImageDescriptor> for Box<[u8]> {
	fn from(desc: &ImageDescriptor) -> Self {
		let mut vec = vec![];

		vec.push(0x2C); // Image Seperator
		vec.extend_from_slice(&desc.left.to_le_bytes());
		vec.extend_from_slice(&desc.top.to_le_bytes());
		vec.extend_from_slice(&desc.width.to_le_bytes());
		vec.extend_from_slice(&desc.height.to_le_bytes());
		vec.push(desc.packed.raw);

		vec.into_boxed_slice()
	}
}

impl From<[u8; 9]> for ImageDescriptor {
	fn from(arr: [u8; 9]) -> Self {
		let left = u16::from_le_bytes(arr[0..2].try_into().unwrap());
		let top = u16::from_le_bytes(arr[2..4].try_into().unwrap());
		let width = u16::from_le_bytes(arr[4..6].try_into().unwrap());
		let height = u16::from_le_bytes(arr[6..8].try_into().unwrap());
		let packed = arr[8];

		Self {
			left,
			top,
			width,
			height,
			packed: ImagePacked { raw: packed },
		}
	}
}

//TODO: Impl to allow changing the packed field easier
