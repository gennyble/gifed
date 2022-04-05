use std::{
	borrow::Cow,
	convert::{TryFrom, TryInto},
	error::Error,
	fmt,
	fs::File,
	io::{BufRead, BufReader, Read},
	path::Path,
};

use crate::{
	block::{
		extension::{Application, GraphicControl},
		Block, ColorTable, CompressedImage, ImageDescriptor, IndexedImage, ScreenDescriptor,
		Version,
	},
	color, Gif,
};

pub struct GifReader {}

impl GifReader {
	pub fn file<P: AsRef<Path>>(path: P) -> Result<Gif, DecodingError> {
		let mut file = File::open(path)?;
		let mut reader = SmartReader {
			inner: vec![],
			position: 0,
		};
		file.read_to_end(&mut reader.inner)?;

		let mut gif = Self::read_required(&mut reader)?;

		if gif.screen_descriptor.color_table_present() {
			let gct_size = gif.screen_descriptor.color_table_len() * 3;
			gif.global_color_table = Some(Self::read_color_table(&mut reader, gct_size)?);
		}

		loop {
			match Self::read_block(&mut reader)? {
				Some(block) => gif.blocks.push(block),
				None => return Ok(gif),
			}
		}
	}

	fn read_required(reader: &mut SmartReader) -> Result<Gif, DecodingError> {
		let version = match reader.take_lossy_utf8(6).as_deref() {
			Some("GIF87a") => Version::Gif87a,
			Some("GIF89a") => Version::Gif89a,
			_ => return Err(DecodingError::UnknownVersionString),
		};

		let mut lsd_buffer: [u8; 7] = [0; 7];
		reader
			.read_exact(&mut lsd_buffer)
			.ok_or(DecodingError::UnexpectedEof)?;

		let lsd = ScreenDescriptor::from(lsd_buffer);

		Ok(Gif {
			header: version,
			screen_descriptor: lsd,
			global_color_table: None,
			blocks: vec![],
		})
	}

	fn read_color_table(
		reader: &mut SmartReader,
		size: usize,
	) -> Result<ColorTable, DecodingError> {
		let buffer = reader
			.take(size as usize)
			.ok_or(DecodingError::UnexpectedEof)?;

		// We get the size from the screen descriptor. This should never return Err
		Ok(ColorTable::try_from(&buffer[..]).unwrap())
	}

	fn read_block(reader: &mut SmartReader) -> Result<Option<Block>, DecodingError> {
		let block_id = reader.u8().ok_or(DecodingError::UnexpectedEof)?;

		//TODO: remove panic
		match block_id {
			0x21 => Self::read_extension(reader).map(|block| Some(block)),
			0x2C => Self::read_image(reader).map(|block| Some(block)),
			0x3B => Ok(None),
			_ => panic!(
				"Unknown block identifier {:X} {:X}",
				block_id, reader.position
			),
		}
	}

	fn read_extension(reader: &mut SmartReader) -> Result<Block, DecodingError> {
		let extension_id = reader.u8().expect("File ended early");

		match extension_id {
			0xF9 => {
				reader.skip(1); // Skip block length, we know it
				let mut data = [0u8; 4];
				reader
					.read_exact(&mut data)
					.ok_or(DecodingError::UnexpectedEof)?;
				reader.skip(1); // Skip block terminator

				Ok(Block::GraphicControlExtension(GraphicControl::from(data)))
			}
			0xFE => Ok(Block::CommentExtension(
				reader.take_and_collapse_subblocks(),
			)),
			0x01 => todo!(), //TODO: do; plain text extension
			0xFF => {
				//TODO: error instead of unwraps
				assert_eq!(Some(11), reader.u8());
				let identifier = TryInto::try_into(reader.take(8).unwrap()).unwrap();
				let authentication_code: [u8; 3] =
					TryInto::try_into(reader.take(3).unwrap()).unwrap();
				let data = reader.take_and_collapse_subblocks();

				Ok(Block::ApplicationExtension(Application {
					identifier,
					authentication_code,
					data,
				}))
			}
			_ => panic!("Unknown Extension Identifier!"),
		}
	}

	fn read_image(mut reader: &mut SmartReader) -> Result<Block, DecodingError> {
		let mut buffer = [0u8; 9];
		reader
			.read_exact(&mut buffer)
			.ok_or(DecodingError::UnexpectedEof)?;
		let descriptor = ImageDescriptor::from(buffer);

		let color_table = if descriptor.has_color_table() {
			let size = descriptor.color_table_size() * 3;
			Some(Self::read_color_table(&mut reader, size)?)
		} else {
			None
		};

		let lzw_csize = reader.u8().ok_or(DecodingError::UnexpectedEof)?;

		let compressed_data = reader.take_and_collapse_subblocks();

		let mut decompress = weezl::decode::Decoder::new(weezl::BitOrder::Lsb, lzw_csize);
		//TODO: remove unwrap
		let mut decompressed_data = decompress.decode(&compressed_data).unwrap();

		Ok(Block::IndexedImage(IndexedImage {
			image_descriptor: descriptor,
			local_color_table: color_table,
			indicies: decompressed_data,
		}))
	}
}

#[derive(Debug)]
pub enum DecodingError {
	IoError(std::io::Error),
	UnknownVersionString,
	UnexpectedEof,
	ColorIndexOutOfBounds,
}

impl Error for DecodingError {}
impl fmt::Display for DecodingError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			DecodingError::IoError(error) => write!(f, "{}", error),
			DecodingError::UnknownVersionString => {
				write!(f, "File did not start with a valid header")
			}
			DecodingError::UnexpectedEof => {
				write!(f, "Found the end of the data at a weird spot")
			}
			DecodingError::ColorIndexOutOfBounds => {
				write!(
					f,
					"The image contained an index not found in the color table"
				)
			}
		}
	}
}

impl From<std::io::Error> for DecodingError {
	fn from(ioerror: std::io::Error) -> Self {
		DecodingError::IoError(ioerror)
	}
}

struct SmartReader {
	inner: Vec<u8>,
	position: usize,
}

impl SmartReader {
	pub fn u8(&mut self) -> Option<u8> {
		self.position += 1;
		self.inner.get(self.position - 1).map(|b| *b)
	}

	pub fn u16(&mut self) -> Option<u16> {
		self.position += 2;
		self.inner
			.get(self.position - 2..self.position)
			.map(|bytes| u16::from_le_bytes(bytes.try_into().unwrap()))
	}

	pub fn skip(&mut self, size: usize) {
		self.position += size;
	}

	pub fn take(&mut self, size: usize) -> Option<&[u8]> {
		self.position += size;
		self.inner.get(self.position - size..self.position)
	}

	//TODO: Result not Option when buffer len
	pub fn read_exact(&mut self, buf: &mut [u8]) -> Option<()> {
		if self.position + buf.len() > self.inner.len() {
			None
		} else {
			self.position += buf.len();
			buf.copy_from_slice(&self.inner[self.position - buf.len()..self.position]);
			Some(())
		}
	}

	pub fn take_vec(&mut self, size: usize) -> Option<Vec<u8>> {
		self.position += size;
		self.inner
			.get(self.position - size..self.position)
			.map(|bytes| bytes.to_vec())
	}

	pub fn take_lossy_utf8(&mut self, size: usize) -> Option<Cow<'_, str>> {
		self.take(size).map(|bytes| String::from_utf8_lossy(bytes))
	}

	pub fn take_data_subblocks(&mut self) -> Vec<Vec<u8>> {
		let mut blocks = vec![];

		loop {
			let block_size = self.u8().expect("Failed to read length of sublock");

			if block_size == 0 {
				return blocks;
			}

			let block = self
				.take_vec(block_size as usize)
				.expect("Failed to read sublock");

			blocks.push(block);
		}
	}

	pub fn take_and_collapse_subblocks(&mut self) -> Vec<u8> {
		let blocks = self.take_data_subblocks();
		let mut ret = vec![];
		for block in blocks {
			ret.extend_from_slice(&block)
		}

		ret
	}
}
