mod gifbuilder;
mod imagebuilder;

use std::fmt;

pub use gifbuilder::GifBuilder;
pub use imagebuilder::ImageBuilder;

pub enum EncodeError {
	InvalidCodeSize { lzw_code_size: u8 },
}

impl fmt::Display for EncodeError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			//TODO: better error
			EncodeError::InvalidCodeSize { lzw_code_size } => {
				write!(f, "InvalidCodeSize => {lzw_code_size}")
			}
		}
	}
}
