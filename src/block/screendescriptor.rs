pub struct ScreenDescriptor {
	pub width: u16,
	pub height: u16,
	pub packed: u8,
	pub background_color_index: u8,
	pub pixel_aspect_ratio: u8
}

impl ScreenDescriptor {
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
	//TODO: Setter for color resolution in packed field
}

impl From<&ScreenDescriptor> for Box<[u8]> {
	fn from(lsd: &ScreenDescriptor) -> Self {
		let mut vec = vec![];
		vec.extend_from_slice(&lsd.width.to_le_bytes());
		vec.extend_from_slice(&lsd.height.to_le_bytes());
		vec.push(lsd.packed);
		vec.push(lsd.background_color_index);
		vec.push(lsd.pixel_aspect_ratio);

		vec.into_boxed_slice()
	}
}