use std::convert::TryFrom;

#[cfg(feature = "colorsquash")]
use colorsquash::Squasher;

use crate::{
	block::{Palette, ScreenDescriptor, Version},
	writer::ImageBuilder,
	Color, EncodeError, Gif,
};

pub struct GifBuilder {
	width: u16,
	height: u16,
	framerate: Option<u16>,
	global_palette: Option<Palette>,
	frames: Vec<Frame>,
}

impl GifBuilder {
	pub fn set_resolution(&mut self, width: u16, height: u16) {
		self.width = width;
		self.height = height;
	}
	pub fn set_framerate(&mut self, framerate: u16) {
		self.framerate = Some(framerate)
	}
	pub fn add_frame(&mut self, frame: Frame) {
		self.frames.push(frame)
	}
	pub fn add_global_palette(&mut self, palette: Palette) {
		self.global_palette = Some(palette)
	}
	pub fn build(self) -> Result<Gif, EncodeError> {
		let Self {
			width,
			height,
			framerate,
			frames,
			global_palette,
		} = self;

		let descriptor = ScreenDescriptor::new(width, height);
		let mut gif = Gif {
			version: Version::Gif89a,
			descriptor,
			palette: global_palette,
			blocks: vec![],
		};

		let images = frames.into_iter().map(|frame| {
			let Frame {
				interval,
				image,
				palette,
			} = frame;

			let delay = interval
				.map(|interval| interval * 10)
				.or(framerate.map(|fr| 100 / fr))
				.unwrap_or(10);
			let image_indicies = image
				.into_iter()
				.flat_map(|row| {
					row.into_iter().map(|c| {
						//if there is a palette for this frame, use that to encode the
						palette
							.or(global_palette)
							.map(|p| p.from_color(c)) //TODO: this is wrong. don't do this
							.flatten()
					})
				})
				.collect::<Vec<_>>();
			let mut ib = ImageBuilder::new(width, height).delay(delay);
			if let Some(p) = palette {
				ib = ib.palette(p);
			}
			ib.build(image_indicies)?.image.compress(None)
		});

		for compressed_image in images {
			match compressed_image {
				Ok(img) => gif.push(img),
				Err(e) => return Err(e),
			}
		}

		Ok(gif)
	}
}

impl Default for GifBuilder {
	fn default() -> Self {
		Self {
			width: 256,
			height: 256,
			framerate: Some(15),
			frames: vec![],
		}
	}
}

pub struct Frame {
	image: Vec<Vec<Color>>, //row-major
	interval: Option<u16>,
	///in hundredths of a second
	palette: Option<Palette>,
}

impl From<Vec<Vec<Color>>> for Frame {
	fn from(image: Vec<Vec<Color>>) -> Self {
		Self {
			image,
			interval: None,
			palette: None,
		}
	}
}

impl Frame {
	#[cfg(feature = "colorsquash")]
	pub fn optimize_palette(&mut self) {
		#[cfg(feature = "rgb")]
		let image_bytes = self.image.clone().into_iter().flatten().collect::<Vec<_>>();
		#[cfg(not(feature = "rgb"))]
		let image_bytes = self
			.image
			.iter()
			.flat_map(|row| row.iter().flat_map(|color| [color.r, color.g, color.b]))
			.collect::<Vec<_>>();
		#[cfg(feature = "rgb")]
		let squasher = Squasher::new(255u8, image_bytes.as_slice());
		#[cfg(not(feature = "rgb"))]
		let squasher = Squasher::new_raw(255u8, image_bytes.as_slice());
		let pal = Palette::try_from(squasher.palette_bytes().as_slice()).unwrap();
		self.set_palette(pal)
	}
	pub fn set_palette(&mut self, palette: Palette) {
		self.palette = Some(palette)
	}
	pub fn set_interval(&mut self, interval_hundredths: u16) {
		self.interval = Some(interval_hundredths);
	}
}
