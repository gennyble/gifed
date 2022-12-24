mod gifbuilder;
mod imagebuilder;

use std::{error::Error, fmt, io::Write};

pub use gifbuilder::GifBuilder;
pub use imagebuilder::ImageBuilder;

use crate::block::{encode_block, Block, LoopCount, Palette, ScreenDescriptor, Version};

use self::gifbuilder::EncodeImage;

pub struct Writer<W: Write> {
	writer: W,
	global_palette: Option<Palette>,
}

impl<W: Write> Writer<W> {
	/// Write the required bits of a GIF. The version written to the stream is
	/// [Version::Gif89a]. If for some reason you need [Version::Gif87a] I
	/// recommend you use [Writer::from_parts].
	pub fn new(
		writer: W,
		width: u16,
		height: u16,
		global_palette: Option<Palette>,
	) -> Result<Self, EncodeError> {
		let mut screen_descriptor = ScreenDescriptor::new(width, height);
		screen_descriptor.set_color_table_metadata(global_palette.as_ref());

		Self::from_parts(writer, Version::Gif89a, screen_descriptor, global_palette)
	}

	//FIXME: gen- This name sucks
	/// Create a new [Writer] from the provided required blocks.
	pub fn from_parts(
		writer: W,
		version: Version,
		screen_descriptor: ScreenDescriptor,
		global_palette: Option<Palette>,
	) -> Result<Self, EncodeError> {
		let mut this = Self {
			writer,
			global_palette,
		};
		this.write_all(version.as_bytes())?;
		this.write_all(&screen_descriptor.as_bytes())?;

		if let Some(palette) = this.global_palette.as_ref() {
			this.write_all(&palette.as_bytes())?;
		}

		Ok(this)
	}

	fn write_all(&mut self, buf: &[u8]) -> Result<(), EncodeError> {
		self.writer
			.write_all(buf)
			.map_err(|error| EncodeError::IoError { error })
	}

	pub fn block(&mut self, block: Block) -> Result<(), EncodeError> {
		self.write_all(&encode_block(&block))
	}

	pub fn repeat(&mut self, count: LoopCount) -> Result<(), EncodeError> {
		self.write_all(&encode_block(&Block::LoopingExtension(count)))
	}

	pub fn image<I: Into<EncodeImage>>(&mut self, image: I) -> Result<(), EncodeError> {
		match image.into() {
			EncodeImage::CompressedImage(compressed) => self.write_all(&compressed.as_bytes()),
			EncodeImage::IndexedImage(indexed) => {
				let lzw_code_size = self.global_palette.as_ref().map(|p| p.lzw_code_size());

				let compressed = indexed.compress(lzw_code_size)?;
				self.write_all(&compressed.as_bytes())
			}
			EncodeImage::BuiltImage(built) => {
				if let Some(gce) = built.gce {
					self.write_all(&encode_block(&Block::GraphicControlExtension(gce)))?;
				}

				let lzw_code_size = self.global_palette.as_ref().map(|p| p.lzw_code_size());

				let compressed = built.image.compress(lzw_code_size)?;
				self.write_all(&compressed.as_bytes())
			}
		}
	}

	pub fn done(mut self) -> Result<(), EncodeError> {
		self.write_all(&[0x3B])
	}
}

#[derive(Debug)]
pub enum EncodeError {
	IoError { error: std::io::Error },
	TooManyColors,
	IndicieSizeMismatch { expected: usize, got: usize },
	InvalidCodeSize { lzw_code_size: u8 },
}

impl Error for EncodeError {}
impl fmt::Display for EncodeError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::IoError { error } => {
				write!(f, "{error}")
			}
			Self::TooManyColors => write!(f, "A palette is limited to 256 colors"),
			Self::IndicieSizeMismatch { expected, got } => {
				write!(f, "Expected to have {expected} indicies but got {got}")
			}
			Self::InvalidCodeSize { lzw_code_size } => {
				write!(f, "InvalidCodeSize => {lzw_code_size}")
			}
		}
	}
}

impl From<std::io::Error> for EncodeError {
	fn from(error: std::io::Error) -> Self {
		EncodeError::IoError { error }
	}
}
