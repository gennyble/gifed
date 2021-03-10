mod colortable;
pub mod extension;
mod indexedimage;
mod imagedescriptor;
mod screendescriptor;
mod version;

pub use colortable::ColorTable;
pub use indexedimage::IndexedImage;
pub use imagedescriptor::ImageDescriptor;
pub use screendescriptor::ScreenDescriptor;
pub use version::Version;

pub enum Block {
	IndexedImage(IndexedImage),
	Extension(extension::Extension)
}