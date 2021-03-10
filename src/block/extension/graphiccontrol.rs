pub struct GraphicControl {
	pub(crate) packed: u8,
	pub(crate) delay_time: u16,
	pub(crate) transparency_index: u8
}

impl GraphicControl {
	pub fn new(disposal_method: DisposalMethod, user_input_flag: bool, transparency_flag: bool, delay_time: u16, transparency_index: u8) -> Self {
		let mut ret = Self {
			packed: 0,
			delay_time,
			transparency_index
		};

		ret.disposal_method(disposal_method);
		ret.user_input(user_input_flag);
		ret.transparency(transparency_flag);

		ret
	}

	pub fn disposal_method(&mut self, method: DisposalMethod) {
		match method {
			DisposalMethod::Clear => self.packed &= 0b111_000_1_1,
			DisposalMethod::DoNotDispose => self.packed |= 0b000_100_0_0,
			DisposalMethod::RestoreBackground => self.packed |= 0b000_010_0_0,
			DisposalMethod::RestorePrevious => self.packed |= 0b000_110_0_0
		};
	}

	pub fn user_input(&mut self, flag: bool) {
		if flag {
			self.packed |= 0b000_000_1_0;
		} else {
			self.packed &= 0b111_111_0_1;
		}
	}

	pub fn transparency(&mut self, flag: bool) {
		if flag {
			self.packed |= 0b000_000_0_1;
		} else {
			self.packed &= 0b111_111_1_0;
		}
	}

	pub fn delay_time(&mut self, hundreths: u16) {
		self.delay_time = hundreths;
	}

	//TODO: Transparency index setter
}

pub enum DisposalMethod {
	Clear,
	DoNotDispose,
	RestoreBackground,
	RestorePrevious
}