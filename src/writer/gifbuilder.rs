use crate::common::{Color, Version};
use super::OutVec;
use super::ImageBuilder;

pub struct GifBuilder {
	version: Version,
	width: u16,
	height: u16,
	global_color_table: Option<Vec<Color>>,
	background_color_index: u8,
	imagebuilders: Vec<ImageBuilder>
}

impl GifBuilder {
	pub fn new(version: Version, width: u16, height: u16) -> Self {
		Self {
			version,
			width,
			height,
			global_color_table: None,
			background_color_index: 0,
			imagebuilders: vec![]
		}
	}

	pub fn global_color_table(mut self, vec: Vec<Color>) -> Self {
		if vec.len() == 0 || vec.len() > 256 {
			//TODO: Throw error instead of panic
			panic!("GCT has to be less than or 256 colors in size, and at least one");
		}

		self.global_color_table = Some(vec);

		self
	}

	pub fn background_color_index(mut self, ind: u8) -> Self {
		if self.global_color_table.is_none() {
			//TODO: Throw error or let it go by, who knows
			panic!("Setting background color index with noGCT!");
		}

		self.background_color_index = ind;
		self
	}

	pub fn image(mut self, ib: ImageBuilder) -> Self {
		self.imagebuilders.push(ib);
		self
	}

	pub fn write_to_vec(&self) -> Vec<u8> {
		let mut out = OutVec::new();

		self
			.write_header(&mut out)
			.write_logical_screen_descriptor(&mut out)
			.write_global_color_table(&mut out)
			.write_images(&mut out)
			.write_trailer(&mut out);

		out.vec()
	}

	fn write_header(&self, out: &mut OutVec) -> &Self {
		out.push_slice(b"GIF");

		//TODO: Automatically detect version
		match self.version {
			Version::Gif87a => out.push_slice(b"87a"),
			Version::Gif89a => out.push_slice(b"89a")
		};

		self
	}

	fn write_logical_screen_descriptor(&self, out: &mut OutVec) -> &Self {
		out.push_u16(self.width).push_u16(self.height);

		let mut packed: u8 = 0;

		if let Some(gct) = &self.global_color_table {
			packed |= 0b1000_0000; // Set the GCT flag

			// GCT size is calulated by raising two to this number plus one,
			// so we have to work backwards.
			let size = (gct.len() as f32).log2().ceil() - 1f32;
			
			packed |= size as u8;
		}
		//TODO: Color Resolution and Sort Flag fields in packed.
		out.push_u8(packed);

		out.push_u8(self.background_color_index)
			.push_u8(0x00); //TOOD: Allow setting pixel aspect ratio

		self
	}

	fn write_global_color_table(&self, out: &mut OutVec) -> &Self {
		if let Some(gct) = &self.global_color_table {
			out.push_colors(&gct);
		}

		self
	}

	fn write_images(&self, out: &mut OutVec) -> &Self {
		//TODO: Deduplicate color table size code
		let mcs = if let Some(gct) = &self.global_color_table {
			((gct.len() as f32).log2().ceil() - 1f32) as u8
		} else {
			0
		};

		for ib in &self.imagebuilders {
			let image = ib.write_to_vec(mcs);
			out.push_slice(&image);
		}

		self
	}

	fn write_trailer(&self, out: &mut OutVec) {
		/*
			"This block is a single-field block indicating the end of the GIF Data Stream. 
			It contains the fixed value 0x3B."
		*/
		out.push_u8(0x3B);
	}
}

#[cfg(test)]
pub mod gifbuilder_test {
	use super::*;

	#[test]
	pub fn writer_header() {
		let mut out = OutVec::new();
		GifBuilder::new(Version::Gif87a, 0, 0).write_header(&mut out);

		assert_eq!(out.vec(), b"GIF87a");
	}

	#[test]
	pub fn write_logical_screen_descriptor() {
		let mut out = OutVec::new();
		GifBuilder::new(Version::Gif87a, 4, 4).write_logical_screen_descriptor(&mut out);

		assert_eq!(out.vec(), vec![0x04, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00]);

		let mut out = OutVec::new();
		let gct = vec![
			Color::new(0, 0, 0), Color::new(0, 0, 0), Color::new(0, 0, 0),
			Color::new(0, 0, 0), Color::new(0, 0, 0)
		];
		GifBuilder::new(Version::Gif87a, 4, 4).global_color_table(gct).write_logical_screen_descriptor(&mut out);

		assert_eq!(out.vec(), vec![0x04, 0x00, 0x04, 0x00, 0b1000_0010, 0x00, 0x00]);
	}

	#[test]
	pub fn write_global_color_table() {
		let mut out = OutVec::new();
		let gct = vec![
			Color::new(1, 2, 3), Color::new(253, 254, 255)
		];
		GifBuilder::new(Version::Gif87a, 4, 4).global_color_table(gct).write_global_color_table(&mut out);

		assert_eq!(out.vec(), vec![1, 2, 3, 253, 254, 255]);
	}

	#[test]
	fn write_images() {
		let gct = vec![
			Color::new(1, 2, 3), Color::new(253, 254, 255)
		];
		let colortable = vec![
			Color::new(0, 0, 0), Color::new(128, 0, 255)
		];
		let indicies = vec![0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0];

		let expected_out = vec![
			0x2C, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x04, 0x00, 0b1000_0000, // Image Descriptor 1
			0, 0, 0, 128, 0, 255, // Color Table
			0x02, 0x05, 0x84, 0x1D, 0x81, 0x7A, 0x50, 0x00, // Image Data 1
			0x2C, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x04, 0x00, 0b0000_0000, // Image Descriptor 2
			0x02, 0x05, 0x84, 0x1D, 0x81, 0x7A, 0x50, 0x00 // Image Data 2
		];

		let mut out = OutVec::new();
		GifBuilder::new(Version::Gif87a, 4, 4)
			.global_color_table(gct)
			.image(ImageBuilder::new(4, 4)
				.color_table(colortable)
				.indicies(indicies.clone())
			).image(ImageBuilder::new(4, 4)
				.indicies(indicies)
			).write_images(&mut out);

		assert_eq!(out.vec(), expected_out);
	}

	#[test]
	fn write_to_vec() {
		let gct = vec![
			Color::new(1, 2, 3), Color::new(253, 254, 255)
		];
		let colortable = vec![
			Color::new(0, 0, 0), Color::new(128, 0, 255)
		];
		let indicies = vec![0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0];

		let expected_out = vec![
			0x47, 0x49, 0x46, 0x38, 0x37, 0x61, // Version - GIF87a
			0x04, 0x00, 0x04, 0x00, 0b1000_0000, 0x00, 0x00, // Logical Screen Descriptor
			1, 2, 3, 253, 254, 255, // Global Color Table
			0x2C, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x04, 0x00, 0b1000_0000, // Image Descriptor 1
			0, 0, 0, 128, 0, 255, // Color Table
			0x02, 0x05, 0x84, 0x1D, 0x81, 0x7A, 0x50, 0x00, // Image Data 1
			0x2C, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x04, 0x00, 0b0000_0000, // Image Descriptor 2
			0x02, 0x05, 0x84, 0x1D, 0x81, 0x7A, 0x50, 0x00, // Image Data 2
			0x3B // Trailer
		];

		let mut out = OutVec::new();
		let actual_out = GifBuilder::new(Version::Gif87a, 4, 4)
			.global_color_table(gct)
			.image(ImageBuilder::new(4, 4)
				.color_table(colortable)
				.indicies(indicies.clone())
			).image(ImageBuilder::new(4, 4)
				.indicies(indicies)
			).write_to_vec();

		assert_eq!(actual_out, expected_out);
	}
}