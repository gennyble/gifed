use crate::{
	block::{
		extension::{DisposalMethod, GraphicControl},
		packed::ImagePacked,
		ImageDescriptor, IndexedImage, Palette, Version,
	},
	EncodeError,
};

pub struct ImageBuilder {
	left_offset: u16,
	top_offset: u16,
	width: u16,
	height: u16,
	color_table: Option<Palette>,

	delay: u16,
	disposal_method: DisposalMethod,
	transparent_index: Option<u8>,
}

impl ImageBuilder {
	pub fn new(width: u16, height: u16) -> Self {
		Self {
			left_offset: 0,
			top_offset: 0,
			width,
			height,
			color_table: None,
			delay: 0,
			disposal_method: DisposalMethod::NoAction,
			transparent_index: None,
		}
	}

	pub fn offset(mut self, left: u16, top: u16) -> Self {
		self.left_offset = left;
		self.top_offset = top;
		self
	}

	pub fn palette(mut self, table: Palette) -> Self {
		self.color_table = Some(table);
		self
	}

	/// Time to wait, in hundredths of a second, before this image is drawn
	pub fn delay(mut self, hundredths: u16) -> Self {
		self.delay = hundredths;
		self
	}

	pub fn disposal_method(mut self, method: DisposalMethod) -> Self {
		self.disposal_method = method;
		self
	}

	pub fn transparent_index(mut self, index: Option<u8>) -> Self {
		self.transparent_index = index;
		self
	}

	pub fn required_version(&self) -> Version {
		if self.delay > 0
			|| self.disposal_method != DisposalMethod::NoAction
			|| self.transparent_index.is_some()
		{
			Version::Gif89a
		} else {
			Version::Gif87a
		}
	}

	pub fn get_graphic_control(&self) -> Option<GraphicControl> {
		if self.required_version() == Version::Gif89a {
			if let Some(transindex) = self.transparent_index {
				Some(GraphicControl::new(
					self.disposal_method,
					false,
					true,
					self.delay,
					transindex,
				))
			} else {
				Some(GraphicControl::new(
					self.disposal_method,
					false,
					false,
					self.delay,
					0,
				))
			}
		} else {
			None
		}
	}

	pub fn build(self, indicies: Vec<u8>) -> Result<BuiltImage, EncodeError> {
		let expected_len = self.width as usize * self.height as usize;
		if indicies.len() != expected_len {
			return Err(EncodeError::IndicieSizeMismatch {
				expected: expected_len,
				got: indicies.len(),
			});
		}

		let gce = self.get_graphic_control();
		let mut imgdesc = ImageDescriptor {
			left: self.left_offset,
			top: self.top_offset,
			width: self.width,
			height: self.height,
			packed: ImagePacked { raw: 0 }, // Set later
		};

		imgdesc.set_color_table_metadata(self.color_table.as_ref());

		let image = IndexedImage {
			image_descriptor: imgdesc,
			local_color_table: self.color_table,
			indicies,
		};

		Ok(BuiltImage { image, gce })
	}
}

pub struct BuiltImage {
	pub image: IndexedImage,
	pub gce: Option<GraphicControl>,
}
