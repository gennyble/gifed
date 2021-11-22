mod colortable;
pub mod extension;
mod imagedescriptor;
mod indexedimage;
mod screendescriptor;
mod version;

pub use colortable::ColorTable;
pub use imagedescriptor::ImageDescriptor;
pub use indexedimage::CompressedImage;
pub use indexedimage::IndexedImage;
pub use screendescriptor::ScreenDescriptor;
pub use version::Version;

use crate::writer::ImageBuilder;

pub enum Block {
	IndexedImage(IndexedImage),
	Extension(extension::Extension),
}

enum WriteBlock {
	ImageBuilder(ImageBuilder),
	Block(Block),
}
