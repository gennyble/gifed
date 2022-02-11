#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphicPacked {
	pub raw: u8,
}

impl GraphicPacked {
	pub(crate) fn new(packed: u8) -> Self {
		Self { raw: packed }
	}

	pub fn reserved(&self) -> u8 {
		graphic_reserved(self.raw)
	}

	pub fn set_reserved(&mut self, reserved: u8) {
		set_graphic_reserved(&mut self.raw, reserved)
	}

	pub fn disposal_method(&self) -> u8 {
		disposal_method(self.raw)
	}

	pub fn set_disposal_method(&mut self, disposal: u8) {
		set_disposal_method(&mut self.raw, disposal)
	}

	pub fn user_input(&self) -> bool {
		user_input(self.raw)
	}

	pub fn set_user_input(&mut self, flag: bool) {
		set_user_input(&mut self.raw, flag)
	}

	pub fn transparent_color(&self) -> bool {
		transparent_color(self.raw)
	}

	pub fn set_transparent_color(&mut self, flag: bool) {
		set_transparent_flag(&mut self.raw, flag)
	}
}

#[inline]
fn graphic_reserved(packed: u8) -> u8 {
	packed & 0b111_000_0_0 >> 5
}

#[inline]
fn disposal_method(packed: u8) -> u8 {
	packed & 0b000_111_0_0 >> 2
}

#[inline]
fn user_input(packed: u8) -> bool {
	packed & 0b000_000_1_0 > 0
}

#[inline]
fn transparent_color(packed: u8) -> bool {
	packed & 0b000_000_0_1 > 0
}

#[inline]
fn set_graphic_reserved(packed: &mut u8, reserved: u8) {
	// We care about the three least significant bits and we want to shift
	// them so they're at the top, five away. From 000_001_1_1 to 111_000_0_0
	*packed = (reserved & 0b0000_0111) << 5;
}

#[inline]
fn set_disposal_method(packed: &mut u8, disposal: u8) {
	// Care about 3 least significant bits and we want them three from the top
	// from 000_001_1_1 into 000_111_0_0
	*packed = (disposal & 0b0000_0111) << 2;
}

#[inline]
fn set_user_input(packed: &mut u8, flag: bool) {
	if flag {
		*packed |= 0b000_000_1_0;
	} else {
		*packed &= 0b111_111_0_1;
	}
}

#[inline]
fn set_transparent_flag(packed: &mut u8, flag: bool) {
	if flag {
		*packed |= 0b000_000_0_1;
	} else {
		*packed &= 0b111_111_1_0;
	}
}
