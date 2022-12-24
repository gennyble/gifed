use std::{convert::TryInto, fmt, time::Duration};

use crate::block::packed::GraphicPacked;

#[derive(Clone, Debug)]
pub struct GraphicControl {
	pub(crate) packed: GraphicPacked,
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
			packed: GraphicPacked { raw: 0 },
			delay,
			transparency_index,
		};

		ret.set_disposal_method(disposal_method);
		ret.packed.set_user_input(user_input_flag);
		ret.packed.set_transparent_flag(transparency_flag);

		ret
	}

	pub fn packed(&self) -> &GraphicPacked {
		&self.packed
	}

	/// Get the disposal method that should be used for the associated image.
	///
	/// # Returns
	/// This method will return `Some([DisposalMethod])` if the disposal method
	/// is recognized, or None if it was set to a reserved value.
	pub fn disposal_method(&self) -> Option<DisposalMethod> {
		match self.packed.disposal_method() {
			0 => Some(DisposalMethod::NoAction),
			1 => Some(DisposalMethod::DoNotDispose),
			2 => Some(DisposalMethod::RestoreBackground),
			3 => Some(DisposalMethod::RestorePrevious),
			_ => None,
		}
	}

	pub fn set_disposal_method(&mut self, dispose: DisposalMethod) {
		match dispose {
			DisposalMethod::NoAction => self.packed.set_disposal_method(0),
			DisposalMethod::DoNotDispose => self.packed.set_disposal_method(1),
			DisposalMethod::RestoreBackground => self.packed.set_disposal_method(2),
			DisposalMethod::RestorePrevious => self.packed.set_disposal_method(4),
		}
	}

	/// Returns the index that should be replaced by a fully transparent pixel
	/// if the transparency flag is set, or None if it's not set.
	pub fn transparent_index(&self) -> Option<u8> {
		if self.packed.transparent_flag() {
			Some(self.transparency_index)
		} else {
			None
		}
	}

	/// Returns the transparency index regardless if the transparency flag is set.
	/// You probably want [GraphicControl::transparency_index] instead.
	pub fn transparent_index_unchecked(&self) -> u8 {
		self.transparency_index
	}

	/// Sets the transparent index and flips the flag to indicate a transparent
	/// index is present if `index` is `Some`.
	pub fn set_transparent_index(&mut self, index: Option<u8>) {
		self.packed.set_transparent_flag(index.is_some());

		if let Some(index) = index {
			self.transparency_index = index;
		}
	}

	pub fn delay_duration(&self) -> Duration {
		Duration::from_millis(self.delay as u64 * 10)
	}

	pub fn delay(&self) -> u16 {
		self.delay
	}

	pub fn delay_mut(&mut self) -> &mut u16 {
		&mut self.delay
	}

	pub fn user_input(&self) -> bool {
		self.packed.user_input()
	}
}

impl From<[u8; 4]> for GraphicControl {
	fn from(arr: [u8; 4]) -> Self {
		let packed = arr[0];
		let delay = u16::from_le_bytes(arr[1..3].try_into().unwrap());
		let transparency_index = arr[3];

		Self {
			packed: GraphicPacked { raw: packed },
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
