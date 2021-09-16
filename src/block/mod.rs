mod colortable;
pub mod extension;
mod imagedescriptor;
mod indexedimage;
mod screendescriptor;
mod version;

pub use colortable::ColorTable;
pub use imagedescriptor::ImageDescriptor;
pub use indexedimage::BlockedImage;
pub use indexedimage::IndexedImage;
pub use screendescriptor::ScreenDescriptor;
pub use version::Version;

pub enum Block {
    IndexedImage(IndexedImage),
    BlockedImage(BlockedImage),
    Extension(extension::Extension),
}
