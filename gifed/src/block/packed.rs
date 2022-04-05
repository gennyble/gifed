#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphicPacked {
	pub raw: u8,
}

impl GraphicPacked {
	pub(crate) fn new(packed: u8) -> Self {
		Self { raw: packed }
	}

	pub fn reserved(&self) -> u8 {
		self.raw & 0b111_000_0_0 >> 5
	}

	pub fn set_reserved(&mut self, reserved: u8) {
		// We care about the three least significant bits and we want to shift
		// them so they're at the top, five away. From 000_001_1_1 to 111_000_0_0
		self.raw = (reserved & 0b0000_0111) << 5;
	}

	pub fn disposal_method(&self) -> u8 {
		self.raw & 0b000_111_0_0 >> 2
	}

	pub fn set_disposal_method(&mut self, disposal: u8) {
		// Care about 3 least significant bits and we want them three from the top
		// from 000_001_1_1 into 000_111_0_0
		self.raw = (disposal & 0b0000_0111) << 2;
	}

	pub fn user_input(&self) -> bool {
		self.raw & 0b000_000_1_0 > 0
	}

	pub fn set_user_input(&mut self, flag: bool) {
		if flag {
			self.raw |= 0b000_000_1_0;
		} else {
			self.raw &= 0b111_111_0_1;
		}
	}

	pub fn transparent_flag(&self) -> bool {
		self.raw & 0b000_000_0_1 > 0
	}

	pub fn set_transparent_flag(&mut self, flag: bool) {
		if flag {
			self.raw |= 0b000_000_0_1;
		} else {
			self.raw &= 0b111_111_1_0;
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImagePacked {
	pub raw: u8,
}

impl ImagePacked {
	pub(crate) fn new(packed: u8) -> Self {
		Self { raw: packed }
	}

	pub fn color_table(&self) -> bool {
		self.raw & 0b1_0_0_00_000 > 0
	}

	pub fn set_color_table(&mut self, flag: bool) {
		if flag {
			self.raw |= 0b1_0_0_00_000;
		} else {
			self.raw &= 0b0_1_1_11_111;
		}
	}

	pub fn interlaced(&self) -> bool {
		self.raw & 0b0_1_0_00_000 > 0
	}

	pub fn set_interlaced(&mut self, flag: bool) {
		if flag {
			self.raw |= 0b0_1_0_00_000;
		} else {
			self.raw &= 0b1_0_1_11_111;
		}
	}

	pub fn sorted(&self) -> bool {
		self.raw & 0b0_0_1_00_000 > 0
	}

	pub fn set_sorted(&mut self, flag: bool) {
		if flag {
			self.raw |= 0b0_0_1_00_000;
		} else {
			self.raw &= 0b1_1_0_11_111;
		}
	}

	pub fn reserved_87a(&self) -> u8 {
		// There was no sort flag in 87a
		self.raw & 0b0_0_111_000 >> 3
	}

	pub fn set_reserved_87a(&mut self, reserved: u8) {
		// We care about the three least significant bits and we want to shift
		// them so they're three higher. From 0_0_000_111 to 0_0_111_000
		self.raw = (reserved & 0b0000_0111) << 3;
	}

	pub fn reserved_89a(&self) -> u8 {
		self.raw & 0b0_0_0_11_000 >> 3
	}

	pub fn set_reserved_89a(&mut self, reserved: u8) {
		// We care about the two least significant bits and we want to shift
		// them so they're three higher. From 0_0_0_00_011 to 0_0_0_11_000
		self.raw = (reserved & 0b0000_0011) << 3;
	}

	pub fn color_table_size(&self) -> u8 {
		self.raw & 0b0_0_0_00_111
	}

	pub fn set_color_table_size(&mut self, size: u8) {
		// The color table is the least significant already, don't do anything
		// except select the bits
		self.raw = size & 0b0_0_0_00_111;
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenPacked {
	pub raw: u8,
}

impl ScreenPacked {
	pub fn color_table(&self) -> bool {
		self.raw & 0b1_000_0_000 > 0
	}

	pub fn set_color_table(&mut self, flag: bool) {
		if flag {
			self.raw |= 0b1_0_0_00_000;
		} else {
			self.raw &= 0b0_1_1_11_111;
		}
	}

	pub fn color_resolution(&self) -> u8 {
		(self.raw & 0b0_111_0_000) >> 4
	}

	pub fn set_color_resolution(&mut self, resolution: u8) {
		self.raw = (resolution & 0b0000_0111) << 4;
	}

	pub fn sorted(&self) -> bool {
		self.raw & 0b0_000_1_000 > 0
	}

	pub fn set_sorted(&mut self, flag: bool) {
		if flag {
			self.raw |= 0b0_000_1_000;
		} else {
			self.raw &= 0b1_111_0_111;
		}
	}

	pub fn color_table_size(&self) -> u8 {
		self.raw & 0b0_0_0_00_111
	}

	pub fn set_color_table_size(&mut self, size: u8) {
		self.raw = size & 0b0_0_0_00_111;
	}
}
