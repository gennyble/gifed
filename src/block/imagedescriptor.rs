pub struct ImageDescriptor {
	// Image Seperator 0x2C is the first byte //
	pub left: u16,
	pub top: u16,
	pub width: u16,
	pub height: u16,
	pub packed: u8
}

impl ImageDescriptor {
	pub fn color_table_present(&mut self, is_present: bool) {
		if is_present {
			self.packed |= 0b1000_0000;
		} else {
			self.packed &= 0b0111_1111;
		}
	}

	pub fn color_table_size(&mut self, size: u8) {
		// GCT size is calulated by raising two to this number plus one,
		// so we have to work backwards.
		let size = (size as f32).log2().ceil() - 1f32;
		self.packed |= size as u8;
	}

	//TODO: Setter for sort flag in packed field
	//TODO: Setter for interlace flag in packed field
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

//TODO: Impl to allow changing the packed field easier