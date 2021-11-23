use std::{fs::File, io::Write, iter::Peekable, path::Path};

use crate::{
	block::{
		encode_block, extension::GraphicControl, Block, ColorTable, ScreenDescriptor, Version,
	},
	colorimage,
	writer::GifBuilder,
	Color, ColorImage,
};
pub struct Gif {
	pub header: Version,
	pub screen_descriptor: ScreenDescriptor,
	pub global_color_table: Option<ColorTable>,
	pub blocks: Vec<Block>, // Trailer at the end of this struct is 0x3B //
}

impl Gif {
	pub fn builder(width: u16, height: u16) -> GifBuilder {
		GifBuilder::new(width, height)
	}

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

		for block in self.blocks.iter() {
			boxed = encode_block(mcs, block);
			out.extend_from_slice(&*boxed);
		}

		// Write Trailer
		out.push(0x3B);

		out
	}

	pub fn save<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
		File::create(path.as_ref())?.write_all(&self.to_vec())
	}

	pub fn images<'a>(&'a self) -> ImageIterator<'a> {
		ImageIterator {
			gif: self,
			veciter: self.blocks.iter(),
		}
	}
}

pub struct ImageIterator<'a> {
	gif: &'a Gif,
	veciter: std::slice::Iter<'a, Block>,
}

impl<'a> Iterator for ImageIterator<'a> {
	type Item = Image<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		let mut transparent = None;

		let img = loop {
			match self.veciter.next() {
				Some(block) => match block {
					Block::IndexedImage(img) => break img,
					Block::GraphicControlExtension(gce) => {
						if gce.is_transparent() {
							transparent = Some(gce.transparency_index());
						} else {
							transparent = None;
						}
					}
					_ => (),
				},
				None => return None,
			}
		};

		if img.image_descriptor.color_table_present() {
			Some(Image {
				width: img.image_descriptor.width,
				height: img.image_descriptor.height,
				left_offset: img.image_descriptor.left,
				top_offset: img.image_descriptor.top,
				palette: &img.local_color_table.as_ref().unwrap(),
				transparent_index: transparent,
				indicies: &img.indicies,
			})
		} else {
			Some(Image {
				width: img.image_descriptor.width,
				height: img.image_descriptor.height,
				left_offset: img.image_descriptor.left,
				top_offset: img.image_descriptor.top,
				palette: self.gif.global_color_table.as_ref().unwrap(),
				transparent_index: transparent,
				indicies: &img.indicies,
			})
		}
	}
}

pub struct Image<'a> {
	pub width: u16,
	pub height: u16,
	pub left_offset: u16,
	pub top_offset: u16,
	pub palette: &'a ColorTable,
	pub transparent_index: Option<u8>,
	pub indicies: &'a [u8],
}

impl<'a> Image<'a> {
	pub fn rgba(&self) -> Option<Vec<u8>> {
		let mut rgba = vec![0; self.indicies.len() * 4];

		for (image_index, &color_index) in self.indicies.iter().enumerate() {
			match self.transparent_index {
				Some(trans) if trans == color_index => {
					rgba[image_index as usize * 4] = 0;
					rgba[image_index * 4 + 1] = 0;
					rgba[image_index * 4 + 2] = 0;
					rgba[image_index * 4 + 3] = 0;
				}
				_ => {
					if let Some(color) = self.palette.get(color_index) {
						rgba[image_index * 4] = color.r;
						rgba[image_index * 4 + 1] = color.g;
						rgba[image_index * 4 + 2] = color.b;
						rgba[image_index * 4 + 3] = 255;
					} else {
						return None;
					}
				}
			}
		}

		Some(rgba)
	}

	pub fn rgb(&self, transparent_replace: Color) -> Option<Vec<u8>> {
		let mut rgb = vec![0; self.indicies.len() * 3];

		for (image_index, &color_index) in self.indicies.iter().enumerate() {
			match self.transparent_index {
				Some(trans) if trans == color_index => {
					rgb[image_index as usize * 4] = transparent_replace.r;
					rgb[image_index * 3 + 1] = transparent_replace.g;
					rgb[image_index * 3 + 2] = transparent_replace.b;
				}
				_ => {
					if let Some(color) = self.palette.get(color_index) {
						rgb[image_index * 3] = color.r;
						rgb[image_index * 3 + 1] = color.g;
						rgb[image_index * 3 + 2] = color.b;
					} else {
						return None;
					}
				}
			}
		}

		Some(rgb)
	}
}

pub struct FrameIterator<'a> {
	gif: &'a Gif,
	veciter: std::slice::Iter<'a, Block>,
	buffer: Vec<u8>,
}

pub struct Frame {
	pub width: u16,
	pub height: u16,
	pub palette: ColorTable,
	pub transparent_index: Option<u8>,
	pub indicies: Vec<u8>,
	pub delay_after_draw: u16,
	pub user_input_flag: bool,
}

#[cfg(test)]
pub mod gif {
	use std::convert::TryInto;
	use std::io::Write;

	use crate::block::extension::DisposalMethod;
	use crate::writer::{GifBuilder, ImageBuilder};
	use crate::Color;

	#[test]
	fn to_vec_gif87a() {
		let gct = vec![Color::new(1, 2, 3), Color::new(253, 254, 255)];
		let colortable = vec![Color::new(0, 0, 0), Color::new(128, 0, 255)];
		let indicies = vec![0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0];

		let expected_out = vec![
			0x47,
			0x49,
			0x46,
			0x38,
			0x37,
			0x61, // Version - GIF87a
			0x04,
			0x00,
			0x04,
			0x00,
			0b1000_0000,
			0x00,
			0x00, // Logical Screen Descriptor
			1,
			2,
			3,
			253,
			254,
			255, // Global Color Table
			0x2C,
			0x00,
			0x00,
			0x00,
			0x00,
			0x04,
			0x00,
			0x04,
			0x00,
			0b1000_0000, // Image Descriptor 1
			0,
			0,
			0,
			128,
			0,
			255, // Color Table
			0x02,
			0x05,
			0x84,
			0x1D,
			0x81,
			0x7A,
			0x50,
			0x00, // Image Data 1
			0x2C,
			0x00,
			0x00,
			0x00,
			0x00,
			0x04,
			0x00,
			0x04,
			0x00,
			0b0000_0000, // Image Descriptor 2
			0x02,
			0x05,
			0x84,
			0x1D,
			0x81,
			0x7A,
			0x50,
			0x00, // Image Data 2
			0x3B, // Trailer
		];

		let actual = GifBuilder::new(4, 4)
			.palette(gct.try_into().unwrap())
			.image(
				ImageBuilder::new(4, 4)
					.palette(colortable.try_into().unwrap())
					.indicies(&indicies),
			)
			.image(ImageBuilder::new(4, 4).indicies(&indicies));

		let bytes = actual.build().unwrap().to_vec();
		assert_eq!(bytes, expected_out);
	}

	#[test]
	fn to_vec_gif89a() {
		let gct = vec![Color::new(1, 2, 3), Color::new(253, 254, 255)];
		let colortable = vec![Color::new(0, 0, 0), Color::new(128, 0, 255)];
		let indicies = vec![0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0];

		let expected_out = vec![
			71, 73, 70, 56, 57, 97, 4, 0, 4, 0, 128, 0, 0, 1, 2, 3, 253, 254, 255, 33, 249, 4, 8,
			64, 0, 0, 0, 44, 0, 0, 0, 0, 4, 0, 4, 0, 128, 0, 0, 0, 128, 0, 255, 2, 5, 132, 29, 129,
			122, 80, 0, 44, 0, 0, 0, 0, 4, 0, 4, 0, 0, 2, 5, 132, 29, 129, 122, 80, 0, 59,
		];

		let actual_out = GifBuilder::new(4, 4)
			.palette(gct.try_into().unwrap())
			.image(
				ImageBuilder::new(4, 4)
					.palette(colortable.try_into().unwrap())
					.indicies(&indicies)
					.disposal_method(DisposalMethod::RestoreBackground)
					.delay(64),
			)
			.image(ImageBuilder::new(4, 4).indicies(&indicies))
			.build()
			.unwrap()
			.to_vec();

		std::fs::File::create("ah.gif")
			.unwrap()
			.write_all(&actual_out)
			.unwrap();
		std::fs::File::create("ah_hand.gif")
			.unwrap()
			.write_all(&expected_out)
			.unwrap();

		assert_eq!(actual_out, expected_out);
	}
}