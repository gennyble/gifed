use std::io::Write;

use crate::{
	block::{
		packed::ScreenPacked, Block, CompressedImage, IndexedImage, LoopCount, Palette,
		ScreenDescriptor, Version,
	},
	EncodeError, Gif,
};

use super::imagebuilder::BuiltImage;

// We want to be able to gold [IndexedImage] as well as [CompressedImage],
// but [Block] does not allow that, so
enum BuildBlock {
	Indexed(IndexedImage),
	Block(Block),
}

pub struct GifBuilder {
	version: Version,
	width: u16,
	height: u16,
	background_color_index: u8,
	global_color_table: Option<Palette>,
	blocks: Vec<BuildBlock>,
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
		}
	}

	pub fn palette(mut self, palette: Palette) -> Self {
		self.global_color_table = Some(palette);
		self
	}

	pub fn background_index(mut self, ind: u8) -> Self {
		self.background_color_index = ind;
		self
	}

	pub fn block(mut self, block: Block) -> Self {
		self.blocks.push(BuildBlock::Block(block));
		self
	}

	pub fn repeat(mut self, count: LoopCount) -> Self {
		self.blocks
			.push(BuildBlock::Block(Block::LoopingExtension(count)));
		self
	}

	pub fn image<I: Into<EncodeImage>>(mut self, img: I) -> Self {
		match img.into() {
			EncodeImage::CompressedImage(ci) => self
				.blocks
				.push(BuildBlock::Block(Block::CompressedImage(ci))),
			EncodeImage::IndexedImage(ii) => self.blocks.push(BuildBlock::Indexed(ii)),
			EncodeImage::BuiltImage(BuiltImage { image, gce }) => {
				if let Some(gce) = gce {
					self.version = Version::Gif89a;

					self.blocks
						.push(BuildBlock::Block(Block::GraphicControlExtension(gce)));
				}

				self.blocks.push(BuildBlock::Indexed(image));
			}
		}

		self
	}

	pub fn build(self) -> Result<Gif, EncodeError> {
		let mut screen_descriptor = ScreenDescriptor {
			width: self.width,
			height: self.height,
			packed: ScreenPacked { raw: 0 }, // Set later
			background_color_index: self.background_color_index,
			pixel_aspect_ratio: 0, //TODO
		};

		screen_descriptor.set_color_table_metadata(self.global_color_table.as_ref());

		let mut gif = Gif {
			header: self.version,
			screen_descriptor,
			global_color_table: self.global_color_table,
			blocks: vec![],
		};

		let lzw_gct_size = gif.global_color_table.as_ref().map(|ct| ct.lzw_code_size());

		for block in self.blocks {
			match block {
				BuildBlock::Indexed(indexed) => {
					let compressed = indexed.compress(lzw_gct_size)?;
					gif.blocks.push(Block::CompressedImage(compressed));
				}
				BuildBlock::Block(block) => gif.blocks.push(block),
			}
		}

		Ok(gif)
	}
}

pub enum EncodeImage {
	CompressedImage(CompressedImage),
	IndexedImage(IndexedImage),
	BuiltImage(BuiltImage),
}

impl From<CompressedImage> for EncodeImage {
	fn from(ci: CompressedImage) -> Self {
		EncodeImage::CompressedImage(ci)
	}
}

impl From<IndexedImage> for EncodeImage {
	fn from(ii: IndexedImage) -> Self {
		EncodeImage::IndexedImage(ii)
	}
}

impl From<BuiltImage> for EncodeImage {
	fn from(bi: BuiltImage) -> Self {
		EncodeImage::BuiltImage(bi)
	}
}
