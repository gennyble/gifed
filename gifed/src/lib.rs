mod color;
mod colorimage;
mod gif;
mod lzw;

pub mod block;
pub mod reader;
pub mod writer;

use core::fmt;
use std::error::Error;

pub use color::Color;
pub use colorimage::ColorImage;
pub use gif::Gif;
pub use lzw::LZW;

/// Perform the algorithm to get the length of a color table from
/// the value of the packed field. The max value here is 256
pub(crate) fn packed_to_color_table_length(packed: u8) -> usize {
	2usize.pow(packed as u32 + 1)
}

//TODO: Be sure to check that fields in LSD and Img. Desc. that were reserved
//in 87a aren't set if version is 87a, or that we return a warning, etc. Just
//remember about this.
//bottom of page 24 in 89a

#[derive(Clone, Copy, Debug)]
pub enum EncodingError {
	TooManyColors,
	NoColorTable,
	IndicieSizeMismatch { expected: usize, got: usize },
}

impl fmt::Display for EncodingError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::TooManyColors => write!(f, "A palette is limited to 256 colors"),
			Self::NoColorTable => write!(
				f,
				"Refusing to set the background color index when no color table is set!"
			),
			Self::IndicieSizeMismatch { expected, got } => {
				write!(f, "Expected to have {} indicies but got {}", expected, got)
			}
		}
	}
}

impl Error for EncodingError {}
