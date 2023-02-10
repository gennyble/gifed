use std::{
	convert::TryFrom,
	error::Error,
	fmt,
	fs::File,
	io::{BufReader, ErrorKind, Read},
	ops::Range,
	path::Path,
};

use crate::{
	block::{
		extension::{Application, GraphicControl},
		Block, CompressedImage, ImageDescriptor, Palette, ScreenDescriptor, Version,
	},
	Gif,
};

pub struct Decoder<R: Read> {
	reader: SmartReader<R>,
}

impl Decoder<BufReader<File>> {
	pub fn file<P: AsRef<Path>>(path: P) -> Result<Self, DecodeError> {
		let file = File::open(path).map_err(|e| DecodeError::IoError(e))?;
		let buffreader = BufReader::new(file);
		Ok(Decoder::new(buffreader))
	}
}

impl<R: Read> Decoder<R> {
	pub fn new(reader: R) -> Self {
		Self {
			reader: SmartReader::new(reader),
		}
	}

	pub fn read(mut self) -> Result<Reader<R>, DecodeError> {
		let version = self.read_version()?;
		let screen_descriptor = self.read_screen_descriptor()?;

		let palette = if screen_descriptor.has_color_table() {
			Some(
				self.reader
					.read_palette(screen_descriptor.color_table_len())?,
			)
		} else {
			None
		};

		Ok(Reader {
			version,
			screen_descriptor,
			palette,
			reader: self.reader,
			saw_trailer: false,
		})
	}

	pub fn read_all(self) -> Result<Gif, DecodeError> {
		let mut decoder = self.read()?;

		let mut blocks = vec![];
		loop {
			match decoder.block()? {
				Some(block) => blocks.push(block.block),
				None => break,
			}
		}

		Ok(Gif {
			header: decoder.version,
			screen_descriptor: decoder.screen_descriptor,
			global_color_table: decoder.palette,
			blocks,
		})
	}

	fn read_version(&mut self) -> Result<Version, DecodeError> {
		let mut buf = [0; 6];
		self.reader.read_exact(&mut buf)?;

		match buf.as_slice() {
			b"GIF87a" => Ok(Version::Gif87a),
			b"GIF89a" => Ok(Version::Gif89a),
			_ => Err(DecodeError::InvalidVersion),
		}
	}

	fn read_screen_descriptor(&mut self) -> Result<ScreenDescriptor, DecodeError> {
		let mut buf = [0; 7];
		self.reader.read_exact(&mut buf)?;
		Ok(buf.into())
	}
}

pub struct ReadBlock {
	pub offset: Range<usize>,
	pub block: Block,
}

pub struct Reader<R: Read> {
	pub version: Version,
	pub screen_descriptor: ScreenDescriptor,
	pub palette: Option<Palette>,

	reader: SmartReader<R>,
	saw_trailer: bool,
}

impl<R: Read> Reader<R> {
	pub fn width(&self) -> u16 {
		self.screen_descriptor.width
	}

	pub fn height(&self) -> u16 {
		self.screen_descriptor.height
	}

	pub fn block(&mut self) -> Result<Option<ReadBlock>, DecodeError> {
		if self.saw_trailer {
			return Ok(None);
		}

		let before = self.reader.bytes_read;
		let introducer = self.reader.u8()?;

		match introducer {
			0x2C => {
				let mut buf = [0; 9];
				self.reader.read_exact(&mut buf)?;
				let descriptor: ImageDescriptor = buf.into();

				let palette = if descriptor.has_color_table() {
					Some(self.reader.read_palette(descriptor.color_table_size())?)
				} else {
					None
				};

				let lzw_code_size = self.reader.u8()?;
				let data = self.reader.take_data_subblocks()?;
				let after = self.reader.bytes_read;

				Ok(Some(ReadBlock {
					offset: before..after,
					block: Block::CompressedImage(CompressedImage {
						image_descriptor: descriptor,
						local_color_table: palette,
						lzw_code_size,
						blocks: data,
					}),
				}))
			}
			0x21 => {
				let block = self.read_extension()?;

				Ok(Some(ReadBlock {
					offset: before..self.reader.bytes_read,
					block,
				}))
			}
			0x3B => {
				self.saw_trailer = true;

				Ok(None)
			}
			_ => Err(DecodeError::UnknownBlock { byte: introducer }),
		}
	}

	fn read_extension(&mut self) -> Result<Block, DecodeError> {
		let label = self.reader.u8()?;

		match label {
			0xF9 => {
				// Graphics Control Extension
				let _len = self.reader.u8()?;
				let mut buf = [0; 4];
				self.reader.read_exact(&mut buf)?;
				let _ = self.reader.u8()?;
				let gce = GraphicControl::from(buf);

				Ok(Block::GraphicControlExtension(gce))
			}
			0xFE => {
				// Comment Extension
				let data = self.reader.take_and_collapse_subblocks()?;
				Ok(Block::CommentExtension(data))
			}
			0xFF => {
				//TODO: Should we check this is 11?
				let _len = self.reader.u8()?;
				let mut app_id = [0; 8];
				let mut auth = [0; 3];
				self.reader.read_exact(&mut app_id)?;
				self.reader.read_exact(&mut auth)?;
				let data = self.reader.take_and_collapse_subblocks()?;
				let app = Application {
					identifier: app_id,
					authentication_code: auth,
					data,
				};

				Ok(Block::ApplicationExtension(app))
			}
			_ => Err(DecodeError::UnknownExtension),
		}
	}
}

struct SmartReader<R: Read> {
	inner: R,
	bytes_read: usize,
}

impl<R: Read> SmartReader<R> {
	pub fn new(reader: R) -> Self {
		Self {
			inner: reader,
			bytes_read: 0,
		}
	}

	pub fn u8(&mut self) -> Result<u8, DecodeError> {
		let mut buffer = [0];

		match self.inner.read(&mut buffer) {
			Ok(read) => {
				self.bytes_read += read;
				Ok(buffer[0])
			}
			Err(e) => Err(DecodeError::IoError(e)),
		}
	}

	#[allow(dead_code)]
	pub fn u16(&mut self) -> Result<u16, DecodeError> {
		let mut buffer = [0, 0];

		self.read_exact(&mut buffer).map(|_| {
			self.bytes_read += 2;
			u16::from_le_bytes(buffer)
		})
	}

	//TODO: Result not Option when buffer len
	pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), DecodeError> {
		match self.inner.read_exact(buf) {
			Ok(_) => {
				self.bytes_read += buf.len();
				Ok(())
			}
			Err(e) if e.kind() == ErrorKind::UnexpectedEof => Err(DecodeError::UnexpectedEof),
			Err(e) => Err(DecodeError::IoError(e)),
		}
	}

	pub fn take_data_subblocks(&mut self) -> Result<Vec<Vec<u8>>, DecodeError> {
		let mut blocks = vec![];

		loop {
			let block_size = self.u8()?;

			if block_size == 0 {
				return Ok(blocks);
			}

			let mut block = vec![0; block_size as usize];
			self.read_exact(&mut block)?;

			blocks.push(block);
		}
	}

	pub fn take_and_collapse_subblocks(&mut self) -> Result<Vec<u8>, DecodeError> {
		let blocks = self.take_data_subblocks()?;
		let mut ret = vec![];
		for block in blocks {
			ret.extend_from_slice(&block)
		}

		Ok(ret)
	}

	pub fn read_palette(&mut self, count: usize) -> Result<Palette, DecodeError> {
		let mut buf = vec![0; count as usize * 3];
		self.read_exact(&mut buf)?;

		Ok(Palette::try_from(buf.as_slice()).unwrap())
	}
}

#[derive(Debug)]
pub enum DecodeError {
	IoError(std::io::Error),
	UnknownVersionString,
	UnexpectedEof,
	ColorIndexOutOfBounds,
	InvalidVersion,
	UnknownBlock { byte: u8 },
	UnknownExtension,
}

impl Error for DecodeError {}
impl fmt::Display for DecodeError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			DecodeError::IoError(error) => write!(f, "{}", error),
			DecodeError::UnknownVersionString => {
				write!(f, "File did not start with a valid header")
			}
			DecodeError::UnexpectedEof => {
				write!(f, "Found the end of the data at a weird spot")
			}
			DecodeError::ColorIndexOutOfBounds => {
				write!(
					f,
					"The image contained an index not found in the color table"
				)
			}
			DecodeError::InvalidVersion => {
				write!(f, "GIF header was incorrect")
			}
			DecodeError::UnknownBlock { byte } => {
				//TODO: gen- Better error message
				write!(f, "No block with introducer {byte:02X}")
			}
			DecodeError::UnknownExtension => {
				//TODO: gen- Better error message
				write!(f, "Unknown extension")
			}
		}
	}
}

impl From<std::io::Error> for DecodeError {
	fn from(ioerror: std::io::Error) -> Self {
		DecodeError::IoError(ioerror)
	}
}
