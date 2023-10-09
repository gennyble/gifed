use std::{fs::File, io::Write, path::Path, time::Duration};

use crate::{
	block::{
		encode_block,
		extension::{DisposalMethod, GraphicControl},
		Block, CompressedImage, IndexedImage, Palette, ScreenDescriptor, Version,
	},
	writer::GifBuilder,
};

#[derive(Clone, Debug)]
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

	pub fn width(&self) -> usize {
		self.screen_descriptor.width as usize
	}

	pub fn height(&self) -> usize {
		self.screen_descriptor.height as usize
	}

	pub fn background_color(&self) -> Option<u8> {
		// vii) Background Color Index - If the Global Color Table Flag is set
		// to (zero), this field should be zero and should be ignored.
		if self.screen_descriptor.has_color_table() {
			Some(self.screen_descriptor.background_color_index)
		} else {
			None
		}
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
