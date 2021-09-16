mod color;
mod gif;
mod lzw;

pub mod block;
pub mod reader;
pub mod writer;

pub use color::Color;
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
