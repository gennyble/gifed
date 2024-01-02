use crate::{
	block::{Palette, ScreenDescriptor, Version},
	writer::ImageBuilder,
	EncodeError, Gif,
};

use color_quant::NeuQuant;
use rgb::{ComponentBytes, FromSlice, RGB8};

use std::convert::TryFrom;

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
				image_indices,
				palette,
			} = frame;

			let delay = interval
				.map(|interval| interval)
				.or(framerate.map(|fr| 100 / fr))
				.unwrap_or(10);
			ImageBuilder::new(width, height)
				.delay(delay)
				.palette(palette)
				.build(image_indices)
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
			global_palette: None,
		}
	}
}

pub struct Frame {
	///indices into the palette
	image_indices: Vec<u8>,
	///in hundredths of a second
	interval: Option<u16>,
	palette: Palette,
}

impl From<Vec<Vec<RGB8>>> for Frame {
	/// image: row-major ordering
	fn from(image: Vec<Vec<RGB8>>) -> Self {
		let flat = image.concat();

		let flat_rgba = flat.as_rgba();
		let quant = NeuQuant::new(1, 256, &flat_rgba.as_bytes());

		let mut indicies = vec![0; flat.len()];
		for (image_idx, px) in flat.iter().enumerate() {
			let color_idx = quant.index_of(&[px.r, px.g, px.b, 255]);
			indicies[image_idx] = color_idx as u8;
		}

		let palette = Palette::try_from(quant.color_map_rgb().as_slice()).unwrap();

		Self {
			image_indices: indicies,
			interval: None,
			palette,
		}
	}
}

impl Frame {
	///
	pub fn set_interval(&mut self, interval_hundredths: u16) {
		self.interval = Some(interval_hundredths);
	}
}
