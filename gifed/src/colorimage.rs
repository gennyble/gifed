use std::convert::TryFrom;

use crate::{block::ColorTable, gif::Image, reader::DecodingError, Color};

pub struct ColorImage {
	width: u16,
	height: u16,
	data: Vec<Pixel>,
}

impl ColorImage {
	pub(crate) fn from_indicies(
		width: u16,
		height: u16,
		indicies: &[u8],
		table: &ColorTable,
		transindex: Option<u8>,
	) -> Result<Self, DecodingError> {
		let mut data = vec![Pixel::Transparent; (width * height) as usize];

		for (image_index, color_index) in indicies.into_iter().enumerate() {
			if let Some(trans) = transindex {
				if trans == *color_index {
					data[image_index] = Pixel::Transparent;
				}
			} else {
				data[image_index] = Pixel::Color(
					table
						.get(*color_index)
						.ok_or(DecodingError::ColorIndexOutOfBounds)?,
				);
			}
		}

		Ok(ColorImage {
			width,
			height,
			data,
		})
	}
}

impl<'a> TryFrom<Image<'a>> for ColorImage {
	type Error = DecodingError;

	fn try_from(img: Image<'a>) -> Result<Self, Self::Error> {
		ColorImage::from_indicies(
			img.width,
			img.height,
			img.indicies,
			img.palette,
			img.transparent_index(),
		)
	}
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Pixel {
	Color(Color),
	Transparent,
}
