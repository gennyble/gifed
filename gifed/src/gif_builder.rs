use crate::{block::Palette, writer::ImageBuilder, Color, EncodeError, Gif};

use color_quant::NeuQuant;
use rgb::{ComponentBytes, FromSlice};

use std::convert::TryFrom;

pub struct VideoGif {
	width: u16,
	height: u16,
	framerate: Option<u16>,
	frames: Vec<Frame>,
}

impl VideoGif {
	pub fn new(width: u16, height: u16) -> Self {
		Self {
			width,
			height,
			framerate: None,
			frames: vec![],
		}
	}

	/// Set the approximate frames per second.
	///
	/// This struct uses a constant framerate and is only precise to hundreths
	/// of a second, so you might not get exactly what you want.
	pub fn set_framerate(&mut self, framerate: u16) {
		self.framerate = Some(100 / framerate);
	}

	pub fn add_frame<F: Into<Frame>>(&mut self, frame: F) {
		self.frames.push(frame.into())
	}

	#[rustfmt::skip] // it was doing things i did not like
	pub fn build(self) -> Result<Gif, EncodeError> {
		let Self { width, height, framerate, frames } = self;

		let mut gif = Gif::new(width, height);

		for Frame { image_indices, interval, palette } in frames {
			//TODO: return error instead of defaulting to 10? or print warning?
			// printing in a library is bad but perhaps so is assuming 10 fps?
			let delay = interval.or(framerate).unwrap_or(10);

			gif.push(
				ImageBuilder::new(width, height)
					.delay(delay)
					.palette(palette)
					.build(image_indices)?,
			)
		}

		Ok(gif)
	}
}

pub struct Frame {
	///indices into the palette
	image_indices: Vec<u8>,
	///in hundredths of a second
	interval: Option<u16>,
	palette: Palette,
}

impl From<&[Color]> for Frame {
	fn from(flat: &[Color]) -> Self {
		let flat_rgba = flat.as_rgba();
		let quant = NeuQuant::new(1, 256, flat_rgba.as_bytes());

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

impl From<(&[Color], u16)> for Frame {
	fn from(image_delay: (&[Color], u16)) -> Self {
		let (flat, delay) = image_delay;
		let mut this: Frame = flat.into();
		this.interval = Some(delay);
		this
	}
}

impl Frame {
	pub fn set_interval(&mut self, interval_hundredths: u16) {
		self.interval = Some(interval_hundredths);
	}
}
