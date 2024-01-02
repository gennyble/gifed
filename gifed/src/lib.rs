mod gif;
mod lzw;

pub mod block;
#[cfg(all(feature = "rgb"))]
pub mod gif_builder;
pub mod reader;
pub mod writer;

pub use reader::DecodeError;
pub use writer::EncodeError;

pub use gif::{Gif, Image};
pub use lzw::LZW;

#[cfg(feature = "rgb")]
pub type Color = rgb::RGB8;
#[cfg(not(feature = "rgb"))]
mod color;
#[cfg(not(feature = "rgb"))]
pub use color::Color;

/// Perform the algorithm to get the length of a color table from
/// the value of the packed field. The max value here is 256
pub(crate) fn packed_to_color_table_length(packed: u8) -> usize {
	1 << (packed + 1)
}

pub(crate) fn color_table_len_to_packed(len: usize) -> u8 {
	((len as f32).log2().ceil() - 1f32) as u8
}

//TODO: Be sure to check that fields in LSD and Img. Desc. that were reserved
//in 87a aren't set if version is 87a, or that we return a warning, etc. Just
//remember about this.
//bottom of page 24 in 89a
