mod color;
mod gif;
mod lzw;

pub mod block;
pub mod writer;

pub use color::Color;
pub use gif::Gif;
pub use lzw::LZW;

//TODO: Be sure to check that fields in LSD and Img. Desc. that were reserved
//in 87a aren't set if version is 87a, or that we return a warning, etc. Just
//remember about this.
//bottom of page 24 in 89a