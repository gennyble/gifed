use crate::block::{ColorTable, IndexedImage, ImageDescriptor};

pub struct ImageBuilder {
	left_offset: u16,
	top_offset: u16,
	width: u16,
	height: u16,
	color_table: Option<ColorTable>,
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

	pub fn color_table(mut self, table: ColorTable) -> Self {
		self.color_table = Some(table);

		self
	}

	pub fn indicies(mut self, vec: Vec<u8>) -> Self {
		self.indicies = vec;
		self
	}

	pub fn build(self) -> IndexedImage {
		let mut imgdesc = ImageDescriptor {
			left: self.left_offset,
			top: self.top_offset,
			width: self.width,
			height: self.height,
			packed: 0 // Set later
		};

		if let Some(lct) = &self.color_table {
			imgdesc.color_table_present(true);
			imgdesc.color_table_size(lct.packed_len());
		}

		IndexedImage {
			image_descriptor: imgdesc,
			local_color_table: self.color_table,
			indicies: self.indicies
		}
	}
}
