use std::{fs::File, io::Write, path::Path, time::Duration};

use crate::block::{
	encode_block,
	extension::{DisposalMethod, GraphicControl},
	Block, CompressedImage, IndexedImage, Palette, ScreenDescriptor, Version,
};

#[derive(Clone, Debug)]
pub struct Gif {
	/// Usually [Version::Gif89a], but might be [Version::Gif87a] for very
	/// simple images.
	pub version: Version,
	pub descriptor: ScreenDescriptor,
	pub palette: Option<Palette>,
	pub blocks: Vec<Block>, // Trailer at the end of this struct is 0x3B //
}

impl Gif {
	pub fn set_width(&mut self, width: u16) {
		self.descriptor.width = width;
	}

	pub fn width(&self) -> u16 {
		self.descriptor.width
	}

	pub fn set_height(&mut self, height: u16) {
		self.descriptor.height = height;
	}

	pub fn height(&self) -> u16 {
		self.descriptor.height
	}

	pub fn set_background_color(&mut self, idx: u8) {
		self.descriptor.background_color_index = idx;
	}

	pub fn background_color(&self) -> Option<u8> {
		// vii) Background Color Index - Index into the Global Color Table for
		// the Background Color. The Background Color is the color used for
		// those pixels on the screen that are not covered by an image. If the
		// Global Color Table Flag is set to (zero), this field should be zero
		// and should be ignored.
		if self.descriptor.has_color_table() {
			Some(self.descriptor.background_color_index)
		} else {
			None
		}
	}

	pub fn as_bytes(&self) -> Vec<u8> {
		let mut out = vec![];

		out.extend_from_slice(self.version.as_bytes());
		out.extend_from_slice(&self.descriptor.as_bytes());

		if let Some(gct) = &self.palette {
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

	/// An iterator over the discrete images in the gif.
	pub fn images(&self) -> ImageIterator<'_> {
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
			let block = self.gif.blocks.get(self.block_index)?;
			if let Block::CompressedImage(img) = block {
				// Step over this image so we don't hit it next time
				self.block_index += 1;

				break img;
			}

			self.block_index += 1;
		};

		Some(Image {
			compressed: img,
			global_palette: self.gif.palette.as_ref(),
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
			.and_then(|gce| gce.transparent_index())
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
			//FIXME: Maybe don't panic here.
			// images can lack a palette entirely. in that case it's up to the
			// decoder to pick one.
			self.global_palette.unwrap()
		}
	}

	/// Make a tRNS block for PNG files.
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
		//FIXME: remove unwrap
		self.compressed.clone().decompress().unwrap()
	}
}

pub enum FrameControl {
	Delay(Duration),
	Input,
	InputOrDelay(Duration),
}
