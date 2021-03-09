use crate::block::{ColorTable, Image, ScreenDescriptor, Version};

pub struct Gif {
	pub header: Version,
	pub screen_descriptor: ScreenDescriptor,
	pub global_color_table: Option<ColorTable>,
	pub images: Vec<Image>
	// Trailer at the end of this struct is 0x3B //
}

impl Gif {
	pub fn to_vec(&self) -> Vec<u8> {
		let mut out = vec![];

		out.extend_from_slice((&self.header).into());
		
		let mut boxed: Box<[u8]> = (&self.screen_descriptor).into();
		out.extend_from_slice(&*boxed);

		// While we output the color table, grab it's length to use when
		// outputting the image, or 0 if we don't have a GCT
		let mcs = if let Some(gct) = &self.global_color_table {
			boxed = gct.into();
			out.extend_from_slice(&*boxed);

			gct.packed_len()
		} else {
			0
		};

		for image in self.images.iter() {
			boxed = image.as_boxed_slice(mcs);
			out.extend_from_slice(&*boxed);
		}

		// Write Trailer
		out.push(0x3B);

		out
	}
}

#[cfg(test)]
pub mod gif {
	use crate::Color;
	use crate::writer::{GifBuilder, ImageBuilder};
	use super::*;

	#[test]
	fn to_vec() {
		let gct = vec![
			Color::new(1, 2, 3), Color::new(253, 254, 255)
		];
		let colortable = vec![
			Color::new(0, 0, 0), Color::new(128, 0, 255)
		];
		let indicies = vec![0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0];

		let expected_out = vec![
			0x47, 0x49, 0x46, 0x38, 0x37, 0x61, // Version - GIF87a
			0x04, 0x00, 0x04, 0x00, 0b1000_0000, 0x00, 0x00, // Logical Screen Descriptor
			1, 2, 3, 253, 254, 255, // Global Color Table
			0x2C, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x04, 0x00, 0b1000_0000, // Image Descriptor 1
			0, 0, 0, 128, 0, 255, // Color Table
			0x02, 0x05, 0x84, 0x1D, 0x81, 0x7A, 0x50, 0x00, // Image Data 1
			0x2C, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x04, 0x00, 0b0000_0000, // Image Descriptor 2
			0x02, 0x05, 0x84, 0x1D, 0x81, 0x7A, 0x50, 0x00, // Image Data 2
			0x3B // Trailer
		];

		let actual_out = GifBuilder::new(Version::Gif87a, 4, 4)
			.global_color_table(gct.into())
			.image(ImageBuilder::new(4, 4)
				.color_table(colortable.into())
				.indicies(indicies.clone())
			).image(ImageBuilder::new(4, 4)
				.indicies(indicies)
			).build().to_vec();

		assert_eq!(actual_out, expected_out);
	}
}