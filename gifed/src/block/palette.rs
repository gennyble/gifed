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

	pub fn packed_len(&self) -> u8 {
		crate::color_table_len_to_packed(self.len())
	}

	pub fn lzw_code_size(&self) -> u8 {
		let table_log = (self.table.len() as f32).log2() as u8;
		table_log.max(2)
	}

	/// Returns the number of colours in the pallette
	pub fn len(&self) -> usize {
		self.table.len()
	}

	pub fn is_empty(&self) -> bool {
		self.len() > 0
	}

	/// Returns the number of items that the decoder *thinks* is in the palette.
	/// This is 2^(n + 1) where n = [Palette::packed_len]
	pub fn computed_len(&self) -> usize {
		1 << (self.packed_len() + 1)
	}

	/// Pushes a color on to the end of the table
	pub fn push(&mut self, color: Color) {
		self.table.push(color);
	}

	pub fn get(&self, index: u8) -> Option<Color> {
		self.table.get(index as usize).copied()
	}

	pub fn from_color(&self, color: Color) -> Option<u8> {
		for (i, &c) in self.table.iter().enumerate() {
			if c == color {
				return Some(i as u8);
			}
		}
		None
	}

	/// How many padding bytes we need to write.
	/// We need to pad the colour table because the size must be a power of two.
	//TODO: gen- better docs
	fn padding(&self) -> usize {
		let comp = self.computed_len();
		(comp - self.len()) * 3
	}

	/// The palette with padding if required
	pub fn as_bytes(&self) -> Vec<u8> {
		let mut bytes = Vec::with_capacity(self.table.len() * 3);
		for color in &self.table {
			bytes.extend_from_slice(&[color.r, color.g, color.b]);
		}

		bytes.extend(std::iter::repeat(0u8).take(self.padding()));

		bytes
	}
}

impl Default for Palette {
	fn default() -> Self {
		Self::new()
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

impl PartialEq for Palette {
	fn eq(&self, other: &Self) -> bool {
		if self.len() != other.len() {
			return false;
		}

		for color in &other.table {
			if !self.table.contains(color) {
				return false;
			}
		}

		true
	}
}

//TODO: TryFrom Vec<u8> (must be multiple of 3 len) and From Vec<Color>
impl TryFrom<&[u8]> for Palette {
	type Error = ();

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		if value.len() % 3 != 0 {
			Err(())
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

#[cfg(test)]
mod test {
	use super::*;

	fn vec_tuple_test(vec: Vec<(u8, u8, u8)>, expected: &[u8]) {
		let plt: Palette = vec.try_into().unwrap();
		let bytes = plt.as_bytes();

		assert_eq!(expected, bytes.as_slice())
	}

	#[test]
	fn writes_one_with_padding() {
		vec_tuple_test(vec![(1, 2, 3)], &[1, 2, 3, 0, 0, 0])
	}

	#[test]
	fn writes_two_without_padding() {
		vec_tuple_test(vec![(1, 2, 3), (4, 5, 6)], &[1, 2, 3, 4, 5, 6])
	}

	fn test_n_with_padding(real_count: usize, exected_padding_bytes: usize) {
		let mut palette = Palette::new();
		let mut expected = vec![];

		for x in 0..real_count {
			let x = x as u8;
			palette.push(Color { r: x, g: x, b: x });
			expected.extend_from_slice(&[x, x, x])
		}

		// yes, this is really how I'm doing it. I have... trust issues with
		// myself and iter::repeat. i hope you understand
		for _ in 0..exected_padding_bytes {
			expected.push(0x00);
		}

		let bytes = palette.as_bytes();
		assert_eq!(expected, bytes.as_slice())
	}

	fn test_n_with_padding_range(real_count_low: u8, real_count_high: u8, next_padstop: usize) {
		for x in real_count_low..=real_count_high {
			test_n_with_padding(x as usize, (next_padstop - x as usize) * 3)
		}
	}

	#[test]
	fn writes_three_with_padding() {
		test_n_with_padding(3, 3);
	}

	#[test]
	fn writes_four_without_padding() {
		test_n_with_padding(4, 0);
	}

	#[test]
	fn writes_five_to_seven_with_padding() {
		test_n_with_padding_range(5, 7, 8);
	}

	#[test]
	fn writes_eight_without_padding() {
		test_n_with_padding(8, 0);
	}

	#[test]
	fn writes_nine_to_fifteen_with_padding() {
		test_n_with_padding_range(9, 15, 16);
	}

	#[test]
	fn writes_sixteen_without_padding() {
		test_n_with_padding(16, 0);
	}

	#[test]
	fn writes_seventeen_to_thirtyone_with_padding() {
		test_n_with_padding_range(17, 31, 32);
	}

	#[test]
	fn writes_thirtytwo_without_padding() {
		test_n_with_padding(32, 0);
	}

	#[test]
	fn writes_thirtythree_to_sixtythree_with_padding() {
		test_n_with_padding_range(33, 63, 64);
	}

	#[test]
	fn writes_sixtyfour_without_padding() {
		test_n_with_padding(64, 0);
	}

	#[test]
	fn writes_sixtyfive_to_onehundredtwentyseven_with_padding() {
		test_n_with_padding_range(65, 127, 128);
	}

	#[test]
	fn writes_onetwentyeight_without_padding() {
		test_n_with_padding(128, 0);
	}

	#[test]
	fn writes_onetwentynine_to_twofiftyfive_with_padding() {
		test_n_with_padding_range(129, 255, 256);
	}

	#[test]
	fn writes_256_without_padding() {
		test_n_with_padding(256, 0);
	}

	#[test]
	fn packed_len_are_correct() {
		let black = Color::new(0, 0, 0);
		let mut palette = Palette::new();

		// Nothing is nothing
		assert_eq!(0, palette.packed_len());

		// One color is still 0 because the formula is
		// 2 ^ (len + 1)
		// which means we should increase at 3
		palette.push(black);
		assert_eq!(0, palette.packed_len());

		palette.push(black);
		assert_eq!(0, palette.packed_len());
	}
}
