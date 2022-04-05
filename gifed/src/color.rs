#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color {
	pub r: u8,
	pub g: u8,
	pub b: u8,
}

impl Color {
	pub fn new(r: u8, g: u8, b: u8) -> Self {
		Self { r, g, b }
	}
}

impl AsRef<Color> for Color {
	fn as_ref(&self) -> &Color {
		self
	}
}

impl From<[u8; 3]> for Color {
	fn from(arr: [u8; 3]) -> Self {
		Self {
			r: arr[0],
			g: arr[1],
			b: arr[2],
		}
	}
}

impl From<(u8, u8, u8)> for Color {
	fn from(t: (u8, u8, u8)) -> Self {
		Self {
			r: t.0,
			g: t.1,
			b: t.2,
		}
	}
}

impl Into<[u8; 3]> for Color {
	fn into(self) -> [u8; 3] {
		[self.r, self.g, self.b]
	}
}

pub type Rgb = Color;

pub struct Rgba {
	pub r: u8,
	pub g: u8,
	pub b: u8,
	pub a: u8,
}
