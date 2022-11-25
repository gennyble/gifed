pub mod extension;
mod imagedescriptor;
mod indexedimage;
pub mod packed;
mod palette;
mod screendescriptor;
mod version;

pub use imagedescriptor::ImageDescriptor;
pub use indexedimage::CompressedImage;
pub use indexedimage::IndexedImage;
pub use palette::Palette;
pub use screendescriptor::ScreenDescriptor;
pub use version::Version;

use crate::writer::ImageBuilder;

use self::extension::Application;
use self::extension::GraphicControl;

pub enum Block {
	CompressedImage(CompressedImage),
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

pub(crate) fn encode_block(mcs: u8, block: &Block) -> Vec<u8> {
	match block {
		Block::CompressedImage(img) => img.as_bytes(),
		Block::GraphicControlExtension(_) => encode_extension(block),
		Block::CommentExtension(_) => encode_extension(block),
		Block::ApplicationExtension(_) => encode_extension(block),
		Block::LoopingExtension(_) => encode_extension(block),
	}
}

fn encode_extension(block: &Block) -> Vec<u8> {
	let mut vec = vec![];
	vec.push(0x21); // Extension Introducer

	match block {
		Block::CompressedImage(_) => unreachable!(),
		Block::GraphicControlExtension(gce) => {
			vec.push(0xF9); // Graphic control label
			vec.push(0x04); // Block size for this extension is always 4
			vec.push(gce.packed.raw);
			vec.extend_from_slice(&gce.delay.to_le_bytes());
			vec.push(gce.transparency_index);
		}
		Block::CommentExtension(comment) => todo!(),
		Block::ApplicationExtension(app) => {
			vec.push(0xFF); // Application extension label
			vec.push(0x0B); // 11 bytes, fixed, for the ident and auth
			vec.extend_from_slice(&app.identifier);
			vec.extend_from_slice(&app.authentication_code);

			for chnk in app.data.chunks(255) {
				vec.push(chnk.len() as u8);
				vec.extend_from_slice(chnk);
			}
		}
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

	// Zero length sub-block indicates end of extension
	vec.push(0x00);

	vec
}
