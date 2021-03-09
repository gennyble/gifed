use crate::block::{ColorTable, ScreenDescriptor, Version};
use crate::writer::ImageBuilder;
use crate::Gif;

pub struct GifBuilder {
	version: Version,
	width: u16,
	height: u16,
	global_color_table: Option<ColorTable>,
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

	pub fn global_color_table(mut self, table: ColorTable) -> Self {
		self.global_color_table = Some(table);

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

	pub fn build(self) -> Gif {
		let mut lsd = ScreenDescriptor {
			width: self.width,
			height: self.height,
			packed: 0, // Set later
			background_color_index: self.background_color_index,
			pixel_aspect_ratio: 0 //TODO: Allow configuring
		};

		if let Some(gct) = &self.global_color_table {
			lsd.color_table_present(true);
			lsd.color_table_size(gct.len() as u8);
		}

		let mut images = vec![];
		for builder in self.imagebuilders.into_iter() {
			images.push(builder.build());
		}

		Gif {
			header: self.version,
			screen_descriptor: lsd,
			global_color_table: self.global_color_table,
			images
		}
	}
}