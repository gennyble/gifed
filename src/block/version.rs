use std::fmt;

pub enum Version {
    Gif87a,
    Gif89a,
}

impl From<&Version> for &[u8] {
    fn from(version: &Version) -> Self {
        match version {
            Version::Gif87a => b"GIF87a",
            Version::Gif89a => b"GIF89a",
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Version::Gif87a => write!(f, "GIF87a"),
            Version::Gif89a => write!(f, "GIF89a"),
        }
    }
}
