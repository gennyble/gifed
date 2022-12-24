use weezl::encode::Encoder;

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
		self.image_descriptor.left
	}

	pub fn width(&self) -> u16 {
		self.image_descriptor.width
	}

	pub fn height(&self) -> u16 {
		self.image_descriptor.height
	}

	/// The `lzw_code_size` should be None if there is a local color table present. If
	/// this image is using the Global Color Table, you must provide an
	/// LZW Minimum Code Size here. It is equal to the value of [Palette::packed_len], but
	/// must also be at least 2.
	pub fn compress(self, lzw_code_size: Option<u8>) -> Result<CompressedImage, EncodeError> {
		// gen- The old code had a +1 here. Why?
		// In the spec, under the section for the Logical Screen Descriptor, it
		// mentions that the size in the packed field is calculated with
		// 2 ^ (packed + 1) and the code size is supposed to be the "number
		// of color bits", which I guess is the exponent?
		let mcs = match self.local_color_table.as_ref() {
			Some(palette) => palette.lzw_code_size(),
			None => match lzw_code_size {
				None => return Err(EncodeError::InvalidCodeSize { lzw_code_size: 0 }),
				Some(mcs) => mcs,
			},
		};

		let mcs = if mcs < 2 { 2 } else { mcs };

		//FIXME: gen- This seems  broken
		//let compressed = LZW::encode(mcs, &self.indicies);
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
	pub fn as_bytes(&self) -> Vec<u8> {
		let mut ret = vec![];

		ret.extend_from_slice(&self.image_descriptor.as_bytes());
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

		let data: Vec<u8> = blocks.into_iter().map(<_>::into_iter).flatten().collect();

		//TODO: remove unwrap
		let mut decompressor = weezl::decode::Decoder::new(weezl::BitOrder::Lsb, lzw_code_size);
		let indicies = decompressor.decode(&data).unwrap();

		Ok(IndexedImage {
			image_descriptor,
			local_color_table,
			indicies,
		})
	}
}
