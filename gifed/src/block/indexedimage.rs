use crate::{reader::DecodeError, EncodeError};

use super::{ImageDescriptor, Palette};

#[derive(Clone, Debug)]
pub struct IndexedImage {
	pub image_descriptor: ImageDescriptor,
	pub local_color_table: Option<Palette>,
	pub indicies: Vec<u8>,
}

impl IndexedImage {
	pub fn left(&self) -> u16 {
		self.image_descriptor.left
	}

	pub fn top(&self) -> u16 {
		self.image_descriptor.top
	}

	pub fn width(&self) -> u16 {
		self.image_descriptor.width
	}

	pub fn height(&self) -> u16 {
		self.image_descriptor.height
	}

	pub fn palette(&self) -> Option<&Palette> {
		self.local_color_table.as_ref()
	}

	/// The `lzw_code_size` should be None if there is a local color table present. If
	/// this image is using the Global Color Table, you must provide an
	/// LZW Minimum Code Size here. It is equal to the value of [Palette::packed_len] + 1 but
	/// must be at least 2.
	pub fn compress(self, lzw_code_size: Option<u8>) -> Result<CompressedImage, EncodeError> {
		let mcs = match self.local_color_table.as_ref() {
			Some(palette) => palette.lzw_code_size(),
			None => match lzw_code_size {
				None => return Err(EncodeError::InvalidCodeSize { lzw_code_size: 0 }),
				Some(mcs) => mcs.max(2),
			},
		};

		#[cfg(not(feature = "weezl-encode"))]
		let compressed = crate::LZW::new(mcs).encode(&self.indicies);

		#[cfg(feature = "weezl-encode")]
		let compressed = Encoder::new(weezl::BitOrder::Lsb, mcs)
			.encode(&self.indicies)
			.unwrap();

		let mut blocks = vec![];
		for chunk in compressed.chunks(255) {
			blocks.push(chunk.to_vec());
		}

		Ok(CompressedImage {
			image_descriptor: self.image_descriptor,
			local_color_table: self.local_color_table,
			lzw_code_size: mcs,
			blocks,
		})
	}
}

#[derive(Clone, Debug)]
pub struct CompressedImage {
	pub image_descriptor: ImageDescriptor,
	pub local_color_table: Option<Palette>,
	pub lzw_code_size: u8,
	pub blocks: Vec<Vec<u8>>,
}

impl CompressedImage {
	pub fn left(&self) -> u16 {
		self.image_descriptor.left
	}

	pub fn top(&self) -> u16 {
		self.image_descriptor.top
	}

	pub fn width(&self) -> u16 {
		self.image_descriptor.width
	}

	pub fn height(&self) -> u16 {
		self.image_descriptor.height
	}

	pub fn palette(&self) -> Option<&Palette> {
		self.local_color_table.as_ref()
	}

	pub fn as_bytes(&self) -> Vec<u8> {
		let mut ret = vec![];

		ret.extend_from_slice(&self.image_descriptor.as_bytes());

		if let Some(palette) = &self.local_color_table {
			ret.extend_from_slice(&palette.as_bytes());
		}

		ret.push(self.lzw_code_size);

		for block in &self.blocks {
			ret.push(block.len() as u8);
			ret.extend_from_slice(block);
		}

		// A zero length block indicates the end of the data stream
		ret.push(0x00);

		ret
	}

	pub fn decompress(self) -> Result<IndexedImage, DecodeError> {
		let CompressedImage {
			image_descriptor,
			local_color_table,
			lzw_code_size,
			blocks,
		} = self;

		let data: Vec<u8> = blocks.into_iter().flat_map(<_>::into_iter).collect();
		let mut decompressor = weezl::decode::Decoder::new(weezl::BitOrder::Lsb, lzw_code_size);
		let indicies = match decompressor.decode(&data) {
			Err(weezl::LzwError::InvalidCode) => Err(DecodeError::LzwInvalidCode),
			Ok(o) => Ok(o),
		}?;

		Ok(IndexedImage {
			image_descriptor,
			local_color_table,
			indicies,
		})
	}
}
