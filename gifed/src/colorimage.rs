use std::{
	convert::TryFrom,
	ops::{Deref, DerefMut, Index},
};

use crate::{block::Palette, color::Rgb, gif::Image, reader::DecodingError, Color};

/// An RGBA, full color image
pub struct RgbaImage {
	pub width: u16,
	pub height: u16,
	pub top: u16,
	pub left: u16,
	pub data: Vec<u8>,
}

impl RgbaImage {
	pub(crate) fn from_indicies(
		width: u16,
		height: u16,
		top: u16,
		left: u16,
		indicies: &[u8],
		table: &Palette,
		transindex: Option<u8>,
	) -> Result<Self, DecodingError> {
		let mut data = vec![0; width as usize * height as usize * 4];

		for (image_index, color_index) in indicies.into_iter().enumerate() {
			if let Some(trans) = transindex {
				if trans == *color_index {
					data[image_index * 4] = 0;
					data[image_index * 4 + 1] = 0;
					data[image_index * 4 + 2] = 0;
					data[image_index * 4 + 3] = 0;
					continue;
				}
			}

			let color = table
				.get(*color_index)
				.ok_or(DecodingError::ColorIndexOutOfBounds)?;

			data[image_index * 4] = color.r;
			data[image_index * 4 + 1] = color.g;
			data[image_index * 4 + 2] = color.b;
			data[image_index * 4 + 3] = 255;
		}

		Ok(Self {
			width,
			height,
			top,
			left,
			data,
		})
	}
}

impl<'a> TryFrom<&Image<'a>> for RgbaImage {
	type Error = DecodingError;

	fn try_from(img: &Image<'a>) -> Result<Self, Self::Error> {
		RgbaImage::from_indicies(
			img.width,
			img.height,
			img.top_offset,
			img.left_offset,
			img.indicies,
			img.palette,
			img.transparent_index(),
		)
	}
}

impl From<RgbImage> for RgbaImage {
	fn from(rgb: RgbImage) -> Self {
		let RgbImage {
			width,
			height,
			top,
			left,
			mut data,
		} = rgb;

		// Extend the data vector to fit the alpha values
		data.extend(std::iter::repeat(0).take(width as usize * height as usize));

		// Work backwards to fill in the alpha values.
		for idx in (0..width as usize * height as usize).rev() {
			data[idx * 4] = data[idx * 3];
			data[idx * 4 + 1] = data[idx * 3 + 1];
			data[idx * 4 + 2] = data[idx * 3 + 2];
			data[idx * 4 + 3] = 255;
		}

		Self {
			width,
			height,
			top,
			left,
			data,
		}
	}
}

impl Deref for RgbaImage {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

impl DerefMut for RgbaImage {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.data.as_mut_slice()
	}
}

/// An RGB, full color image
pub struct RgbImage {
	pub width: u16,
	pub height: u16,
	pub top: u16,
	pub left: u16,
	pub data: Vec<u8>,
}

impl RgbImage {
	pub fn from_image<'a>(
		image: &Image<'a>,
		transparent_replace: Color,
	) -> Result<Self, DecodingError> {
		let mut data = vec![0; image.indicies.len() * 3];

		for (image_idx, &color_idx) in image.indicies.iter().enumerate() {
			match image.transparent_index() {
				Some(trans) if trans == color_idx => {
					data[image_idx as usize * 4] = transparent_replace.r;
					data[image_idx * 3 + 1] = transparent_replace.g;
					data[image_idx * 3 + 2] = transparent_replace.b;
				}
				_ => {
					if let Some(color) = image.palette.get(color_idx) {
						data[image_idx * 3] = color.r;
						data[image_idx * 3 + 1] = color.g;
						data[image_idx * 3 + 2] = color.b;
					} else {
						return Err(DecodingError::ColorIndexOutOfBounds);
					}
				}
			}
		}

		Ok(Self {
			width: image.width,
			height: image.height,
			top: image.top_offset,
			left: image.left_offset,
			data,
		})
	}
}

impl Deref for RgbImage {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

impl DerefMut for RgbImage {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.data.as_mut_slice()
	}
}
