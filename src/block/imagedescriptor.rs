use std::convert::TryInto;

pub struct ImageDescriptor {
	// Image Seperator 0x2C is the first byte //
	pub left: u16,
	pub top: u16,
	pub width: u16,
	pub height: u16,
	pub packed: u8,
}

impl ImageDescriptor {
	pub fn set_color_table_present(&mut self, is_present: bool) {
		if is_present {
			self.packed |= 0b1000_0000;
		} else {
			self.packed &= 0b0111_1111;
		}
	}

	pub fn set_color_table_size(&mut self, size: u8) {
		// GCT size is calulated by raising two to this number plus one,
		// so we have to work backwards.
		let size = (size as f32).log2().ceil() - 1f32;
		self.packed |= size as u8;
	}

	//TODO: Setter for sort flag in packed field
	//TODO: Setter for interlace flag in packed field

	pub fn color_table_present(&self) -> bool {
		self.packed & 0b1000_0000 != 0
	}

	pub fn color_table_size(&self) -> usize {
		crate::packed_to_color_table_length(self.packed & 0b0000_0111)
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
		vec.push(desc.packed);

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
			packed,
		}
	}
}

//TODO: Impl to allow changing the packed field easier
