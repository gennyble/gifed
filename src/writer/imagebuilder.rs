use crate::common::Color;
use super::OutVec;
use super::LZW;

pub struct ImageBuilder {
	left_offset: u16,
	top_offset: u16,
	width: u16,
	height: u16,
	color_table: Option<Vec<Color>>,
	indicies: Vec<u8>
}

impl ImageBuilder {
	pub fn new(width: u16, height: u16) -> Self {
		Self {
			left_offset: 0,
			top_offset: 0,
			width,
			height,
			color_table: None,
			indicies: vec![]
		}
	}

	pub fn offsets(mut self, left_offset: u16, top_offset: u16) -> Self {
		self.left_offset = left_offset;
		self.top_offset = top_offset;
		self
	}

	pub fn left_offset(mut self, offset: u16) -> Self {
		self.left_offset = offset;
		self
	}

	pub fn top_offset(mut self, offset: u16) -> Self {
		self.top_offset = offset;
		self
	}

	pub fn color_table(mut self, vec: Vec<Color>) -> Self {
		if vec.len() == 0 || vec.len() > 256 {
			//TODO: Throw error instead of panic
			panic!("Color table has to be less than or 256 colors in size, and at least one");
		}

		self.color_table = Some(vec);

		self
	}

	pub fn indicies(mut self, vec: Vec<u8>) -> Self {
		self.indicies = vec;
		self
	}

	//TODO: Make lzw_minimum_code_size optional. ONly needed with global color tables
	pub fn write_to_vec(&self, lzw_minimum_code_size: u8) -> Vec<u8> {
		let mut out = OutVec::new();

		self.write_image_descriptor(&mut out)
			.write_color_table(&mut out)
			.write_image_data(&mut out, lzw_minimum_code_size);

		out.vec()
	}

	fn write_image_descriptor(&self, out: &mut OutVec) -> &Self {
		// Image seperator. At the start of every image descriptor
		out.push_u8(0x2C)
		.push_u16(self.left_offset)
		.push_u16(self.top_offset)
		.push_u16(self.width)
		.push_u16(self.height);

		// Taken from gifbuilder.rs
		//TODO: deduplciate code
		let mut packed: u8 = 0;
		if let Some(ct) = &self.color_table {
			packed |= 0b1000_0000; // Set the color table flag

			let size = (ct.len() as f32).log2().ceil() - 1f32;
			
			packed |= size as u8;
		}
		//TODO: Interlace and Sort flags in packed
		out.push_u8(packed);

		self
	}

	fn write_color_table(&self, out: &mut OutVec) -> &Self {
		if let Some(ct) = &self.color_table {
			out.push_colors(&ct);
		}

		self
	}

	fn write_image_data(&self, out: &mut OutVec, minimum_code_size: u8) -> &Self {
		let mut mcs = minimum_code_size;

		//TODO: Deduplicate color table size code
		if let Some(ct) = &self.color_table {
			mcs = ((ct.len() as f32).log2().ceil() - 1f32) as u8;
		}

		if mcs < 2 {
			// Must always be true: 2 <= mcs <= 8
			mcs = 2;
		}

		// First write out the MCS
		out.push_u8(mcs);

		let compressed = LZW::encode(mcs, &self.indicies);
		
		for chunk in compressed.chunks(255) {
			out.push_u8(chunk.len() as u8);
			out.push_slice(chunk);
		}
		// Data block length 0 to indicate an end
		out.push_u8(0x00);

		self
	}
}

#[cfg(test)]
mod imagebuilder_test {
	use super::*;

	#[test]
	fn write_to_vec() {
		let colortable = vec![
			Color::new(0, 0, 0),
			Color::new(128, 0, 255)
		];
		let indicies = vec![0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0];

		let expected_out = vec![
			0x2C, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x04, 0x00, 0b1000_0000, // Image Descriptor
			0, 0, 0, 128, 0, 255, // Color Table
			0x02, 0x05, 0x84, 0x1D, 0x81, 0x7A, 0x50, 0x00 // Image Data
		];
		let actual_out = ImageBuilder::new(4, 4)
			.color_table(colortable)
			.indicies(indicies)
			.write_to_vec(0);

		assert_eq!(actual_out, expected_out);
	}

	#[test]
	fn write_image_descriptor() {
		let mut out = OutVec::new();
		ImageBuilder::new(16, 16).offsets(1, 6).write_image_descriptor(&mut out);

		assert_eq!(out.vec(), vec![0x2C, 0x01, 0x00, 0x06, 0x00, 0x10, 0x00, 0x10, 0x00, 0x00]);

		let mut out = OutVec::new();
		ImageBuilder::new(16, 16)
			.offsets(1, 6)
			.color_table(vec![Color::new(0, 0, 0)])
			.write_image_descriptor(&mut out);

		assert_eq!(out.vec(), vec![0x2C, 0x01, 0x00, 0x06, 0x00, 0x10, 0x00, 0x10, 0x00, 0b1000_0000]);
	}

	#[test]
	fn write_color_table() {
		let mut out = OutVec::new();
		ImageBuilder::new(16, 16)
			.color_table(vec![Color::new(1, 2, 3), Color::new(253, 254, 255)])
			.write_color_table(&mut out);

		assert_eq!(out.vec(), vec![0x01, 0x02, 0x03, 0xFD, 0xFE, 0xFF]);
	}

	#[test]
	fn write_image_data() {
		#[test]
		fn encode() {
			let indicies = vec![0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0];
			let output = vec![0x02, 0x05, 0x84, 0x1D, 0x81, 0x7A, 0x50, 0x00];

			let mut out = OutVec::new();
			ImageBuilder::new(16, 16)
				.indicies(indicies)
				.write_image_data(&mut out, 2);
	
			assert_eq!(out.vec(), output);
		}
	}
}