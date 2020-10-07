use crate::common::Color;

#[derive(Debug, PartialEq)]
pub struct OutVec {
	data: Vec<u8>
}

impl OutVec {
	pub fn new() -> Self {
		Self {
			data: vec![]
		}
	}

	pub fn push_u8(&mut self, n: u8) -> &mut Self {
		self.data.push(n);
		self
	}

	pub fn push_u16(&mut self, n: u16) -> &mut Self {
		self.data.extend_from_slice(&n.to_le_bytes());
		self
	}

	pub fn push_u32(&mut self, n: u32) -> &mut Self {
		self.data.extend_from_slice(&n.to_le_bytes());
		self
	}

	pub fn push_u64(&mut self, n: u64) -> &mut Self {
		self.data.extend_from_slice(&n.to_le_bytes());
		self
	}

	pub fn push_slice(&mut self, slice: &[u8]) -> &mut Self {
		self.data.extend_from_slice(slice);
		self
	}

	pub fn push_color(&mut self, color: &Color) -> &mut Self {
		self.data.extend_from_slice(&[color.r, color.g, color.b]);
		self
	}

	pub fn push_colors(&mut self, colors: &[Color]) -> &mut Self {
		for color in colors {
			self.push_color(color);
		}
		self
	}

	pub fn vec(self) -> Vec<u8> {
		self.data
	}
}

impl From<Vec<u8>> for OutVec {
	fn from(v: Vec<u8>) -> Self {
		let mut outvec = Self::new();
		outvec.push_slice(&v);

		outvec
	}
}