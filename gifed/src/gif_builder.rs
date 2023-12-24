use std::convert::TryFrom;

#[cfg(feature = "colorsquash")]
use colorsquash::{Squasher, SquasherBuilder};

use crate::{
	block::{packed::ScreenPacked, Palette, ScreenDescriptor, Version},
	writer::ImageBuilder,
	Color, Gif,
};

pub struct GifBuilder {
	width: u16,
	height: u16,
	framerate: Option<u16>,
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
	pub fn build(self) -> Gif {
		let Self {
			width,
			height,
			framerate,
			frames,
		} = self;

		let descriptor = ScreenDescriptor::new(width, height);
		let mut gif = Gif {
			version: Version::Gif89a,
			descriptor,
			palette: None,
			blocks: vec![],
		};

		let images = frames.into_iter().map(|frame| {
			let Frame {
				interval,
				image,
				palette,
			} = frame;

			let palette = palette.unwrap();
			let delay = interval
				.map(|interval| interval * 10)
				.or(framerate.map(|fr| 100 / fr))
				.unwrap_or(10);
			let image_bytes = image
				.into_iter()
				.flat_map(|row| {
					row.into_iter().map(|c| {
						palette
							.from_color(c)
							.expect("palette should be able to convert any color")
					})
				})
				.collect::<Vec<_>>();
			let ib = ImageBuilder::new(width, height)
				.delay(delay)
				.build(image_bytes)
				.expect("image building should succeed");
			ib.image.compress(None).expect("compression should succeed")
		});

		for compressed_image in images {
			gif.push(compressed_image);
		}

		gif
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
		let image_bytes = self
			.image
			.iter()
			.flat_map(|row| row.iter().flat_map(|color| [color.r, color.g, color.b]))
			.collect::<Vec<_>>();
		let squasher = SquasherBuilder::default()
			.max_colors(255u8)
			.build(image_bytes.as_slice());
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
