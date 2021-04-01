mod graphiccontrol;

pub use graphiccontrol::{DisposalMethod, GraphicControl};

pub enum Extension {
	GraphicControl(GraphicControl),
	Looping(u16)
	// Comment
	// Plain Text
	// Generic Application
}

impl From<&Extension> for Box<[u8]> {
	fn from(ext: &Extension) -> Self {
		let mut vec = vec![];
		vec.push(0x21); // Push the extension introducer

		match ext {
			Extension::GraphicControl(gc) => {
				vec.push(0xF9); // Graphic control label
				vec.push(0x04); // Block size for this extension is always 4
				vec.push(gc.packed);
				vec.extend_from_slice(&gc.delay_time.to_le_bytes());
				vec.push(gc.transparency_index);
			},
			Extension::Looping(count) => {
				vec.push(0xFF); // Application extension label
				vec.push(0x0B); // 11 bytes in this block
				vec.extend_from_slice(b"NETSCAPE2.0"); // App. ident. and "auth code"
				vec.push(0x03); // Sub-block length
				vec.push(0x01); // Identifies netscape looping extension
				vec.extend_from_slice(&count.to_le_bytes());
			} 
		}

		vec.push(0x00); // Zero-length data block indicates end of extension
		vec.into_boxed_slice()
	}
}