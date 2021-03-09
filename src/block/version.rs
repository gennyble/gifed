pub enum Version {
	Gif87a,
	Gif89a
}

impl From<&Version> for &[u8] {
	fn from(version: &Version) -> Self {
		match version {
			Version::Gif87a => b"GIF87a",
			Version::Gif89a => b"GIF89a"
		}
	}
}