use std::{fs::File, io::Write, path::Path, time::Duration};

use crate::{
	block::{
		encode_block,
		extension::{DisposalMethod, GraphicControl},
		packed::ImagePacked,
		Block, CompressedImage, ImageDescriptor, IndexedImage, Palette, ScreenDescriptor, Version,
	},
	writer::GifBuilder,
};
pub struct Gif {
	pub header: Version,
	pub screen_descriptor: ScreenDescriptor,
	pub global_color_table: Option<Palette>,
	pub blocks: Vec<Block>, // Trailer at the end of this struct is 0x3B //
}

impl Gif {
	pub fn builder(width: u16, height: u16) -> GifBuilder {
		GifBuilder::new(width, height)
	}

	pub fn as_bytes(&self) -> Vec<u8> {
		let mut out = vec![];

		out.extend_from_slice(&self.header.as_bytes());
		out.extend_from_slice(&self.screen_descriptor.as_bytes());

		if let Some(gct) = &self.global_color_table {
			out.extend_from_slice(&gct.as_bytes());
		}

		for block in self.blocks.iter() {
			out.extend_from_slice(&encode_block(block));
		}

		// Write Trailer
		out.push(0x3B);

		out
	}

	pub fn save<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
		File::create(path.as_ref())?.write_all(&self.as_bytes())
	}

	pub fn images<'a>(&'a self) -> ImageIterator<'a> {
		ImageIterator {
			gif: self,
			block_index: 0,
		}
	}
}

pub struct ImageIterator<'a> {
	gif: &'a Gif,
	block_index: usize,
}

impl<'a> Iterator for ImageIterator<'a> {
	type Item = Image<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		let starting_block = self.block_index;

		let img = loop {
			match self.gif.blocks.get(self.block_index) {
				Some(block) => match block {
					Block::CompressedImage(img) => {
						// Step over this image so we don't hit it next time
						self.block_index += 1;

						break img;
					}
					_ => (),
				},
				None => return None,
			}

			self.block_index += 1;
		};

		Some(Image {
			compressed: &img,
			global_palette: self.gif.global_color_table.as_ref(),
			blocks: &self.gif.blocks[starting_block..self.block_index],
		})
	}
}

pub struct Image<'a> {
	pub compressed: &'a CompressedImage,
	pub global_palette: Option<&'a Palette>,
	pub blocks: &'a [Block],
}

impl<'a> Image<'a> {
	pub fn width(&self) -> u16 {
		self.compressed.image_descriptor.width
	}

	pub fn height(&self) -> u16 {
		self.compressed.image_descriptor.height
	}

	pub fn top(&self) -> u16 {
		self.compressed.image_descriptor.top
	}

	pub fn left(&self) -> u16 {
		self.compressed.image_descriptor.left
	}

	pub fn graphic_control(&self) -> Option<&GraphicControl> {
		for block in self.blocks {
			if let Block::GraphicControlExtension(gce) = block {
				return Some(gce);
			}
		}

		None
	}

	pub fn transparent_index(&self) -> Option<u8> {
		self.graphic_control()
			.map(|gce| gce.transparent_index())
			.flatten()
	}

	pub fn frame_control(&self) -> Option<FrameControl> {
		if let Some(gce) = self.graphic_control() {
			let delay = gce.delay_duration();
			let user_input = gce.user_input();

			match (delay.is_zero(), user_input) {
				(true, true) => Some(FrameControl::Input),
				(false, true) => Some(FrameControl::InputOrDelay(delay)),
				(false, false) => Some(FrameControl::Delay(delay)),
				(true, false) => None,
			}
		} else {
			None
		}
	}

	pub fn disposal_method(&self) -> DisposalMethod {
		if let Some(gce) = self.graphic_control() {
			gce.disposal_method().unwrap_or(DisposalMethod::NoAction)
		} else {
			DisposalMethod::NoAction
		}
	}

	pub fn palette(&self) -> &Palette {
		if let Some(plt) = self.compressed.local_color_table.as_ref() {
			plt
		} else {
			//FIXME: Maybe don't panic here
			self.global_palette.unwrap()
		}
	}

	pub fn png_trns(&self) -> Option<Vec<u8>> {
		let palette = self.palette();
		if let Some(trans_idx) = self.transparent_index() {
			let mut trns = Vec::with_capacity(palette.len());

			for idx in 0..palette.len() as u8 {
				if idx == trans_idx {
					trns.push(0u8);
				} else {
					trns.push(255);
				}
			}

			return Some(trns);
		}

		None
	}

	/// Clones the CompressedImage and decompresses it.
	pub fn decompess(&self) -> IndexedImage {
		//FIXME: unwrap
		self.compressed.clone().decompress().unwrap()
	}
}

pub enum FrameControl {
	Delay(Duration),
	Input,
	InputOrDelay(Duration),
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
					.build(indicies.clone())
					.unwrap(),
			)
			.image(ImageBuilder::new(4, 4).build(indicies).unwrap());

		let bytes = actual.build().unwrap().as_bytes();
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
					.disposal_method(DisposalMethod::RestoreBackground)
					.delay(64)
					.build(indicies.clone())
					.unwrap(),
			)
			.image(ImageBuilder::new(4, 4).build(indicies).unwrap())
			.build()
			.unwrap()
			.as_bytes();

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
