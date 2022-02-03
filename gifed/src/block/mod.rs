mod colortable;
pub mod extension;
mod imagedescriptor;
mod indexedimage;
pub mod packed;
mod screendescriptor;
mod version;

pub use colortable::ColorTable;
pub use imagedescriptor::ImageDescriptor;
pub use indexedimage::CompressedImage;
pub use indexedimage::IndexedImage;
pub use screendescriptor::ScreenDescriptor;
pub use version::Version;

use crate::writer::ImageBuilder;

use self::extension::Application;
use self::extension::GraphicControl;

pub enum Block {
	IndexedImage(IndexedImage),
	//TODO: Extension(Extension),
	GraphicControlExtension(GraphicControl),
	CommentExtension(Vec<u8>),
	//TODO: PlainTextExtension(PlainTextExtension),
	ApplicationExtension(Application),
	LoopingExtension(LoopCount),
}

enum WriteBlock {
	ImageBuilder(ImageBuilder),
	Block(Block),
}

pub enum LoopCount {
	Forever,
	Number(u16),
}

pub(crate) fn encode_block(mcs: u8, block: &Block) -> Box<[u8]> {
	match block {
		Block::IndexedImage(img) => img.as_boxed_slice(mcs),
		Block::GraphicControlExtension(_) => encode_extension(block),
		Block::CommentExtension(_) => encode_extension(block),
		Block::ApplicationExtension(_) => encode_extension(block),
		Block::LoopingExtension(_) => encode_extension(block),
	}
}

fn encode_extension(block: &Block) -> Box<[u8]> {
	let mut vec = vec![];
	vec.push(0x21); // Extension Introducer

	match block {
		Block::IndexedImage(_) => unreachable!(),
		Block::GraphicControlExtension(gce) => {
			vec.push(0xF9); // Graphic control label
			vec.push(0x04); // Block size for this extension is always 4
			vec.push(gce.packed);
			vec.extend_from_slice(&gce.delay.to_le_bytes());
			vec.push(gce.transparency_index);
		}
		Block::CommentExtension(comment) => todo!(),
		Block::ApplicationExtension(app) => todo!(),
		Block::LoopingExtension(lc) => {
			vec.push(0xFF); // Application extension label
			vec.push(0x0B); // 11 bytes in this block
			vec.extend_from_slice(b"NETSCAPE2.0"); // App. ident. and "auth code"
			vec.push(0x03); // Sub-block length
			vec.push(0x01); // Identifies netscape looping extension

			match lc {
				LoopCount::Forever => vec.extend_from_slice(&[0x00, 0x00]),
				LoopCount::Number(count) => vec.extend_from_slice(&count.to_le_bytes()),
			}
		}
	}

	vec.push(0x00); // Zero length sub-block indicates end of extension
	vec.into_boxed_slice()
}
