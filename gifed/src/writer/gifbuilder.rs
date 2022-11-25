use std::convert::TryInto;

use crate::block::packed::ScreenPacked;
use crate::block::{Block, LoopCount, Palette, ScreenDescriptor, Version};
use crate::writer::ImageBuilder;
use crate::{EncodingError, Gif};

pub struct GifBuilder {
	version: Version,
	width: u16,
	height: u16,
	background_color_index: u8,
	global_color_table: Option<Palette>,
	blocks: Vec<Block>,
	error: Option<EncodingError>,
}

impl GifBuilder {
	pub fn new(width: u16, height: u16) -> Self {
		Self {
			version: Version::Gif87a,
			width,
			height,
			background_color_index: 0,
			global_color_table: None,
			blocks: vec![],
			error: None,
		}
	}

	pub fn palette(mut self, palette: Palette) -> Self {
		self.global_color_table = Some(palette);
		self
	}

	pub fn background_index(mut self, ind: u8) -> Self {
		if self.error.is_some() {
			return self;
		}

		if self.global_color_table.is_none() {
			self.error = Some(EncodingError::NoColorTable);
		} else {
			self.background_color_index = ind;
		}
		self
	}

	pub fn image(mut self, ib: ImageBuilder) -> Self {
		if self.error.is_some() {
			return self;
		}

		if ib.required_version() == Version::Gif89a {
			self.version = Version::Gif89a;
		}

		if let Some(gce) = ib.get_graphic_control() {
			self.blocks.push(Block::GraphicControlExtension(gce));
		}

		//FIXME
		/*
		match ib.build() {
			Ok(image) => self.blocks.push(Block::IndexedImage(image)),
			Err(e) => self.error = Some(e),
		}*/

		self
	}

	/*pub fn extension(mut self, ext: Extension) -> Self {
		self.blocks.push(Block::Extension(ext));
		self
	}*/

	pub fn repeat(mut self, count: LoopCount) -> Self {
		self.blocks.push(Block::LoopingExtension(count));
		self
	}

	pub fn build(self) -> Result<Gif, EncodingError> {
		if let Some(error) = self.error {
			return Err(error);
		}

		let mut lsd = ScreenDescriptor {
			width: self.width,
			height: self.height,
			packed: ScreenPacked { raw: 0 }, // Set later
			background_color_index: self.background_color_index,
			pixel_aspect_ratio: 0, //TODO: Allow configuring
		};

		if let Some(gct) = &self.global_color_table {
			println!("build {}", gct.len());
			lsd.set_color_table_metadata(Some(gct));
		}

		Ok(Gif {
			header: self.version,
			screen_descriptor: lsd,
			global_color_table: self.global_color_table,
			blocks: self.blocks,
		})
	}
}
