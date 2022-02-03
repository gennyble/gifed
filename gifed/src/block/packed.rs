struct InnerPacked<T> {
	raw: T,
}

impl InnerPacked<&u8> {
	#[inline]
	fn color_table(&self) -> bool {
		self.raw & 0b1000_0000 > 0
	}

	#[inline]
	fn color_resolution(&self) -> u8 {
		(self.raw & 0b0111_0000) >> 4
	}

	#[inline]
	fn logical_sort(&self) -> bool {
		self.raw & 0b0000_1000 > 0
	}

	#[inline]
	fn image_sort(&self) -> bool {
		self.raw & 0b0010_0000 > 0
	}

	#[inline]
	fn color_table_size(&self) -> u8 {
		self.raw & 0b0000_0111
	}

	#[inline]
	fn interlace(&self) -> bool {
		self.raw & 0b0100_0000 > 0
	}
}

impl InnerPacked<&mut u8> {
	#[inline]
	fn set_color_table(&mut self, flag: bool) {
		if flag {
			*self.raw |= 0b1000_0000;
		} else {
			*self.raw &= 0b0111_1111;
		}
	}

	#[inline]
	fn set_color_resolution(&mut self, resolution: u8) {
		*self.raw |= (resolution & 0b0111_0000) << 4;
	}

	#[inline]
	fn set_logical_sort(&mut self, flag: bool) {
		if flag {
			*self.raw |= 0b0000_1000;
		} else {
			*self.raw &= 0b1111_0111;
		}
	}

	#[inline]
	fn set_image_sort(&mut self, flag: bool) {
		if flag {
			*self.raw |= 0b0010_0000;
		} else {
			*self.raw &= 0b1101_1111;
		}
	}

	#[inline]
	fn set_color_table_size(&mut self, size: u8) {
		*self.raw |= size & 0b0000_0111;
	}

	#[inline]
	fn set_interlace(&mut self, flag: bool) {
		if flag {
			*self.raw |= 0b0100_0000;
		} else {
			*self.raw &= 0b1011_1111;
		}
	}
}
