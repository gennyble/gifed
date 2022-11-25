pub use crate::Color;
use crate::EncodeError;
use std::{
	convert::{TryFrom, TryInto},
	ops::Deref,
};

#[derive(Clone, Debug)]
pub struct Palette {
	table: Vec<Color>,
}

impl Palette {
	pub fn new() -> Self {
		Self { table: vec![] }
	}

	/// Returns the number of colors in the color table as used by the packed
	/// fields in the Logical Screen Descriptor and Image Descriptor. You can
	/// get the actual size with the [`len`](struct.ColorTable.html#method.len) method.
	///
	/// This value is equal to `log2([Palette::len]) - 1`. In other words, 2^(n + 1) will
	/// give you the same value as [Palette::len]. (where `n` is the value returned)
	pub fn packed_len(&self) -> u8 {
		((self.table.len() as f32).log2().ceil() - 1f32) as u8
	}

	/// Returns the number of items in the table
	pub fn len(&self) -> usize {
		self.table.len()
	}

	/// Pushes a color on to the end of the table
	pub fn push(&mut self, color: Color) {
		self.table.push(color);
	}

	pub fn get(&self, index: u8) -> Option<Color> {
		self.table.get(index as usize).map(|v| v.clone())
	}

	pub fn from_color<C: AsRef<Color>>(&self, color: C) -> Option<u8> {
		for (i, &c) in self.table.iter().enumerate() {
			if c == *color.as_ref() {
				return Some(i as u8);
			}
		}
		None
	}

	pub fn as_bytes(&self) -> Vec<u8> {
		let mut bytes = Vec::with_capacity(self.table.len() * 3);
		for color in &self.table {
			bytes.extend_from_slice(&[color.r, color.g, color.b]);
		}

		bytes
	}
}

impl Deref for Palette {
	type Target = [Color];

	fn deref(&self) -> &Self::Target {
		&self.table
	}
}

impl AsRef<Palette> for Palette {
	fn as_ref(&self) -> &Palette {
		self
	}
}

//TODO: TryFrom Vec<u8> (must be multiple of 3 len) and From Vec<Color>
impl TryFrom<&[u8]> for Palette {
	type Error = ();

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		if value.len() % 3 != 0 {
			return Err(());
		} else {
			Ok(Self {
				table: value
					.chunks(3)
					.map(|slice| Color::from(TryInto::<[u8; 3]>::try_into(slice).unwrap()))
					.collect::<Vec<Color>>(),
			})
		}
	}
}

impl TryFrom<Vec<Color>> for Palette {
	type Error = EncodeError;

	fn try_from(value: Vec<Color>) -> Result<Self, Self::Error> {
		if value.len() > 256 {
			Err(EncodeError::TooManyColors)
		} else {
			Ok(Self { table: value })
		}
	}
}

impl TryFrom<Vec<(u8, u8, u8)>> for Palette {
	type Error = EncodeError;

	fn try_from(value: Vec<(u8, u8, u8)>) -> Result<Self, Self::Error> {
		if value.len() > 256 {
			Err(EncodeError::TooManyColors)
		} else {
			Ok(Self {
				table: value.into_iter().map(|c| c.into()).collect(),
			})
		}
	}
}