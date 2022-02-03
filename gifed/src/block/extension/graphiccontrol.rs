use std::{convert::TryInto, fmt, time::Duration};

#[derive(Clone, Debug)]
pub struct GraphicControl {
	pub(crate) packed: u8,
	pub(crate) delay: u16,
	pub(crate) transparency_index: u8,
}

impl GraphicControl {
	pub fn new(
		disposal_method: DisposalMethod,
		user_input_flag: bool,
		transparency_flag: bool,
		delay: u16,
		transparency_index: u8,
	) -> Self {
		let mut ret = Self {
			packed: 0,
			delay,
			transparency_index,
		};

		ret.set_disposal_method(disposal_method);
		ret.set_user_input(user_input_flag);
		ret.set_transparent(transparency_flag);

		ret
	}

	/// Get the disposal method that should be used for the associated image.
	///
	/// # Returns
	/// This method will return `Some([DisposalMethod])` if the disposal method
	/// is recognized, or None if it was set to a reserved value.
	pub fn disposal_method(&self) -> Option<DisposalMethod> {
		match self.packed & 0b000_111_00 {
			0b000_000_00 => Some(DisposalMethod::NoAction),
			0b000_100_00 => Some(DisposalMethod::DoNotDispose),
			0b000_010_00 => Some(DisposalMethod::RestoreBackground),
			0b000_110_00 => Some(DisposalMethod::RestorePrevious),
			_ => None,
		}
	}

	/// Set the disposal method that shoudl be used for the associated image.
	pub fn set_disposal_method(&mut self, method: DisposalMethod) {
		match method {
			DisposalMethod::NoAction => self.packed &= 0b111_000_1_1,
			DisposalMethod::DoNotDispose => self.packed |= 0b000_100_0_0,
			DisposalMethod::RestoreBackground => self.packed |= 0b000_010_0_0,
			DisposalMethod::RestorePrevious => self.packed |= 0b000_110_0_0,
		};
	}

	/// Returns the index that should be replaced by a fully transparent pixel
	/// if the transparency flag is set, or None if it's not set.
	pub fn transparent_index(&self) -> Option<u8> {
		if self.transparent() {
			Some(self.transparency_index)
		} else {
			None
		}
	}

	/// Returns the transparency index regardless if the transparency flag is set.
	/// You probably want [GraphicControl::transparency_index] instead.s
	pub fn transparent_index_unchecked(&self) -> u8 {
		self.transparency_index
	}

	/// Sets the transparent index flag to the value provided. This will change
	/// the index value in any way and should be used with caution. You probably
	/// want [GraphicControl::set_transparent_index] instead.
	pub fn set_transparent(&mut self, flag: bool) {
		if flag {
			self.packed |= 0b000_000_0_1;
		} else {
			self.packed &= 0b111_111_1_0;
		}
	}

	/// Sets the transparent index and flips the flag to indicate a transparent
	/// index is present.
	pub fn set_transparent_index(&mut self, index: Option<u8>) {
		self.set_transparent(index.is_some());

		if let Some(index) = index {
			self.transparency_index = index;
		}
	}

	/// Get the value of the transparency flag
	pub fn transparent(&self) -> bool {
		self.packed & 0b000_000_0_1 > 0
	}

	pub fn user_input(&self) -> bool {
		self.packed & 0b000_000_1_0 > 0
	}

	pub fn set_user_input(&mut self, flag: bool) {
		if flag {
			self.packed |= 0b000_000_1_0;
		} else {
			self.packed &= 0b111_111_0_1;
		}
	}

	pub fn delay(&self) -> u16 {
		self.delay
	}

	pub fn delay_duration(&self) -> Duration {
		Duration::from_millis(self.delay as u64 * 10)
	}

	pub fn delay_mut(&mut self) -> &mut u16 {
		&mut self.delay
	}

	pub fn packed(&self) -> u8 {
		self.packed
	}

	pub fn packed_mut(&mut self) -> &mut u8 {
		&mut self.packed
	}
}

impl From<[u8; 4]> for GraphicControl {
	fn from(arr: [u8; 4]) -> Self {
		let packed = arr[0];
		let delay = u16::from_le_bytes(arr[1..3].try_into().unwrap());
		let transparency_index = arr[3];

		Self {
			packed,
			delay,
			transparency_index,
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DisposalMethod {
	NoAction,
	DoNotDispose,
	RestoreBackground,
	RestorePrevious,
}

impl fmt::Display for DisposalMethod {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let st = match self {
			DisposalMethod::NoAction => "Dispose as Normal",
			DisposalMethod::DoNotDispose => "No Dispose",
			DisposalMethod::RestoreBackground => "Restore to background",
			DisposalMethod::RestorePrevious => "Restore previous image",
		};

		write!(f, "{}", st)
	}
}
