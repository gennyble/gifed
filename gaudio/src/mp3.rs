use std::io::{BufRead, BufReader, Cursor, Read};

/// Destroy an MP3, ripping it's frames apart. Also removes any ID3v2 tags
/// because who needs metadata?
pub struct Breaker {
	frames: Vec<Frame>,
}

impl Breaker {
	pub fn new() -> Self {
		Self { frames: vec![] }
	}

	pub fn split(&mut self, mut data: Vec<u8>) -> Result<(), std::io::Error> {
		let cursor = Cursor::new(data);
		let mut reader = BufReader::new(cursor);

		let mut consumed = 0;
		loop {
			let mut three = [0x00, 0x00, 0x00];
			reader.read_exact(&mut three)?;
			consumed += 3;

			if &three == b"ID3" {
				println!("ID3v2 offset {:X}", consumed);
				Self::skip_id3v2(&mut reader)?
			} else if three[0] == 0xFF && three[2] & 0b1110_0000 == 0b1110_0000 {
				let mut one_more = [0x00];
				reader.read_exact(&mut one_more)?;
			}
		}

		todo!()
	}

	/// Assumes the ident "TAG" was already consumed
	fn skip_id3v2<R: BufRead>(reader: &mut R) -> Result<(), std::io::Error> {
		// We don't actually want this, but want to get rid of it.
		let mut version_and_flags = [0x00, 0x00, 0x00];
		reader.read_exact(&mut version_and_flags)?;

		println!(
			"Version {} Revision {}",
			version_and_flags[0], version_and_flags[1]
		);

		let mut syncsafe_size = [0x00, 0x00, 0x00, 0x00];
		reader.read_exact(&mut syncsafe_size)?;

		// Size is MSB
		let mut size = syncsafe_size[3] as u32;
		// Shift right eight, but back one because most significant bit is 0 due to syncsafe
		size |= (syncsafe_size[2] as u32) << 7;
		size |= (syncsafe_size[1] as u32) << 14;
		size |= (syncsafe_size[0] as u32) << 21;

		let human = if size > 1024 * 1024 {
			format!("{:.2}MiB", size as f32 / (1024.0 * 1024.0))
		} else if size > 1024 {
			format!("{:.2}KiB", size as f32 / 1024.0)
		} else {
			format!("{size}B")
		};

		println!("ID3v2 size is {human} bytes");

		// Make a vec size big. We're not here to be efficient, sorry if this dissapoint you.
		let mut skip = vec![0x00; size as usize];
		reader.read_exact(&mut skip)
	}
}

pub struct Frame {
	header: Header,
	data: Vec<u8>,
}

pub struct Header {
	// I only want to parse what i need, but we need this for writing out, so
	raw: [u8; 4],
	version: Version,
	layer: Layer,
	crc: bool,
}

impl Header {
	pub fn from_bytes(raw: [u8; 4]) -> Result<Self, Error> {
		if raw[0] != 0xFF || raw[1] & 0b1110_0000 != 0b1110_0000 {
			return Err(Error::HeaderUnsync);
		}

		let version = Version::from(raw[1]);
		let layer = Layer::from(raw[1]);
		let crc = raw[1] & 1 == 0;

		let bitrate = Bitrate::resolve(raw[2], version, layer);

		//TODO: gen- love, you were trying to get the size of the data field. We need
		//to know the sampling rate and the pad bit for that, which happen to be the
		//next three bits.

		todo!()
	}

	// Algorithm taken from:
	// http://www.multiweb.cz/twoinches/mp3inside.htm
	/// The length of the header and data
	pub fn length(&self) -> usize {
		todo!()
	}

	/// The length of the audio data. This is just the length - 4
	pub fn data_length(&self) -> usize {
		self.length() - 4
	}
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("tried to parse header, but first 11 bits were not 1; not synced!")]
	HeaderUnsync,
	#[error("The version or the layer was a reserved value")]
	CannotResolveBitrate,
	#[error("Bitrate bits were all 1")]
	BitrateBad,
}

#[derive(Copy, Clone, Debug)]
pub enum Version {
	Mpeg2_5,
	Reserved,
	Mpeg2,
	Mpeg1,
}

impl From<u8> for Version {
	fn from(byte: u8) -> Self {
		match byte & 0b000_11_000 {
			0b000_00_000 => Version::Mpeg2_5,
			0b000_01_000 => Version::Reserved,
			0b000_10_000 => Version::Mpeg2,
			0b000_11_000 => Version::Mpeg1,
			_ => unreachable!(),
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub enum Layer {
	Reserved,
	Layer3,
	Layer2,
	Layer1,
}

impl From<u8> for Layer {
	fn from(byte: u8) -> Self {
		match byte & 0b000_00_110 {
			0b000_00_000 => Layer::Reserved,
			0b000_00_010 => Layer::Layer3,
			0b000_00_100 => Layer::Layer2,
			0b000_00_110 => Layer::Layer1,
			_ => unreachable!(),
		}
	}
}

/// What do you want from me it's hard to name a thing that's just number.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Bitrate {
	RateFree,
	Rate8,
	Rate16,
	Rate24,
	Rate32,
	Rate40,
	Rate48,
	Rate56,
	Rate64,
	Rate80,
	Rate96,
	Rate112,
	Rate128,
	Rate144,
	Rate160,
	Rate176,
	Rate192,
	Rate224,
	Rate256,
	Rate288,
	Rate320,
	Rate352,
	Rate384,
	Rate416,
	Rate448,
}

impl Bitrate {
	/// Takes the third byte of the header and other neccesary information
	pub fn resolve(third: u8, version: Version, layer: Layer) -> Result<Self, Error> {
		#[rustfmt::skip]
		macro_rules! v {
			(Any) => { _ };
			(v1) => { Version::Mpeg1 };
			(v2) => { Version::Mpeg2 | Version::Mpeg2_5 };
		}

		#[rustfmt::skip]
		macro_rules! l {
			(Any) => { _ };
			(l1) => { Layer::Layer1 };
			(l2) => { Layer::Layer2 };
			(l3) => { Layer::Layer3 };
			(l23) => { Layer::Layer2 | Layer::Layer3 };
		}

		#[rustfmt::skip]
		macro_rules! br {
			(b0000) => { Fourbit::Zero };
			(b0001) => { Fourbit::One };
			(b0010) => { Fourbit::Two };
			(b0011) => { Fourbit::Three };
			(b0100) => { Fourbit::Four };
			(b0101) => { Fourbit::Five };
			(b0110) => { Fourbit::Six };
			(b0111) => { Fourbit::Seven };
			(b1000) => { Fourbit::Eight };
			(b1001) => { Fourbit::Nine };
			(b1010) => { Fourbit::Ten };
			(b1011) => { Fourbit::Eleven };
			(b1100) => { Fourbit::Twelve };
			(b1101) => { Fourbit::Thirteen };
			(b1110) => { Fourbit::Fourteen };
			(b1111) => { Fourbit::Fifteen };

			(down1 b0010) => {br!(b0011)};
			(down1 b0011) => {br!(b0100)};
			(down1 b0100) => {br!(b0101)};
			(down1 b0101) => {br!(b0110)};
			(down1 b0110) => {br!(b0111)};
			(down1 b0111) => {br!(b1000)};
			(down1 b1000) => {br!(b1001)};

			(down4 b0010) => {br!(b0110)};
			(down4 b0011) => {br!(b0111)};
			(down4 b0100) => {br!(b1000)};
			(down4 b0101) => {br!(b1001)};
			(down4 b0110) => {br!(b1010)};
			(down4 b0111) => {br!(b1011)};
			(down4 b1000) => {br!(b1100)};

			(down5 b0110) => {br!(b1011)};
			(down5 b0111) => {br!(b1100)};
			(down5 b1000) => {br!(b1101)};

			(down6 b0110) => {br!(b1100)};
			(down6 b0111) => {br!(b1101)};
			(down6 b1000) => {br!(b1110)};
		}

		macro_rules! vl1 {
			($br:ident) => {
				brvl!($br v1 l1)
			};
		}

		macro_rules! v2_l23 {
			($br:ident) => {
				brvl!($br v2 l23)
			};
		}

		macro_rules! brvl {
			($br:ident $v:ident $l:ident) => {
				(br!($br), v!($v), l!($l))
			};

			(down1 $br:ident $v:ident $l:ident) => {
				(br!(down1 $br), v!($v), l!($l))
			};

			(down4 $br:ident $v:ident $l:ident) => {
				(br!(down4 $br), v!($v), l!($l))
			};

			(down5 $br:ident $v:ident $l:ident) => {
				(br!(down5 $br), v!($v), l!($l))
			};

			(down6 $br:ident $v:ident $l:ident) => {
				(br!(down6 $br), v!($v), l!($l))
			};
		}

		macro_rules! heartbeat_bird {
			($col_v1_l2:ident) => {
				brvl!($col_v1_l2 v1 l2) | brvl!($col_v1_l2 v2 l1) | brvl!(down1 $col_v1_l2 v1 l3) | brvl!(down4 $col_v1_l2 v2 l23)
			};
		}

		// rustc was apparently unhappy about my brvl!(down4) calls. And down5, down6 :(
		macro_rules! crooked_down {
			($col_v1_l1:ident) => {
				brvl!($col_v1_l1 v1 l1) | (br!(down4 $col_v1_l1), v!(v1), l!(l2)) | (br!(down5 $col_v1_l1), v!(v1), l!(l3)) | (br!(down6 $col_v1_l1), v!(v2), l!(l1))
			};
		}

		let br_4bit = Fourbit::from_u8((third & 0b1111_0000) >> 4).expect("this can't happen");

		let tuple = (br_4bit, version, layer);

		match tuple {
			// These patterns cover a very large surface area
			heartbeat_bird!(b0010) => Ok(Bitrate::Rate48),
			heartbeat_bird!(b0011) => Ok(Bitrate::Rate56),
			heartbeat_bird!(b0100) => Ok(Bitrate::Rate64),
			heartbeat_bird!(b0101) => Ok(Bitrate::Rate80),
			heartbeat_bird!(b0110) => Ok(Bitrate::Rate96),
			heartbeat_bird!(b0111) => Ok(Bitrate::Rate112),
			heartbeat_bird!(b1000) => Ok(Bitrate::Rate128),

			crooked_down!(b0110) => Ok(Bitrate::Rate192),
			crooked_down!(b0111) => Ok(Bitrate::Rate224),
			crooked_down!(b1000) => Ok(Bitrate::Rate256),

			// Then we start at the top and work our way down,
			// row by row as long as there are still values we have to match there
			brvl!(b0000 Any Any) => Ok(Bitrate::RateFree),

			brvl!(b0001 v1 Any) => Ok(Bitrate::Rate32),
			brvl!(b0001 v2 l1) => Ok(Bitrate::Rate32),
			brvl!(b0001 v2 l23) => Ok(Bitrate::Rate8),

			vl1!(b0010) => Ok(Bitrate::Rate64),
			brvl!(b0010 v1 l3) => Ok(Bitrate::Rate40),
			v2_l23!(b0010) => Ok(Bitrate::Rate16),

			vl1!(b0011) => Ok(Bitrate::Rate96),
			v2_l23!(b0011) => Ok(Bitrate::Rate24),

			vl1!(b0100) => Ok(Bitrate::Rate128),
			v2_l23!(b0100) => Ok(Bitrate::Rate32),

			vl1!(b0101) => Ok(Bitrate::Rate160),
			v2_l23!(b0101) => Ok(Bitrate::Rate40),

			// 0110, 0111, and 1000 are entirely covered by the patterns :D
			vl1!(b1001) => Ok(Bitrate::Rate288),
			brvl!(b1001 v1 l2) => Ok(Bitrate::Rate160),
			brvl!(b1001 v2 l1) => Ok(Bitrate::Rate144),

			brvl!(b1010 v1 l1) => Ok(Bitrate::Rate320),
			brvl!(b1010 v1 l3) => Ok(Bitrate::Rate160),
			brvl!(b1010 v2 l1) => Ok(Bitrate::Rate160),

			vl1!(b1011) => Ok(Bitrate::Rate352),
			brvl!(b1011 v2 l1) => Ok(Bitrate::Rate176),

			vl1!(b1100) => Ok(Bitrate::Rate384),

			vl1!(b1101) => Ok(Bitrate::Rate416),
			brvl!(b1101 v1 l2) => Ok(Bitrate::Rate320),
			v2_l23!(b1101) => Ok(Bitrate::Rate144),

			vl1!(b1110) => Ok(Bitrate::Rate448),
			brvl!(b1110 v1 l2) => Ok(Bitrate::Rate384),
			brvl!(b1110 v1 l3) => Ok(Bitrate::Rate320),
			v2_l23!(b1110) => Ok(Bitrate::Rate160),

			(br!(b1111), _, _) => Err(Error::BitrateBad),
			(_, Version::Reserved, _) | (_, _, Layer::Reserved) => Err(Error::CannotResolveBitrate),
		}
	}

	pub fn from_kbps(kbps: usize) -> Option<Self> {
		println!("{kbps}");
		match kbps {
			8 => Some(Bitrate::Rate8),
			16 => Some(Bitrate::Rate16),
			24 => Some(Bitrate::Rate24),
			32 => Some(Bitrate::Rate32),
			40 => Some(Bitrate::Rate40),
			48 => Some(Bitrate::Rate48),
			56 => Some(Bitrate::Rate56),
			64 => Some(Bitrate::Rate64),
			80 => Some(Bitrate::Rate80),
			96 => Some(Bitrate::Rate96),
			112 => Some(Bitrate::Rate112),
			128 => Some(Bitrate::Rate128),
			144 => Some(Bitrate::Rate144),
			160 => Some(Bitrate::Rate160),
			176 => Some(Bitrate::Rate176),
			192 => Some(Bitrate::Rate192),
			224 => Some(Bitrate::Rate224),
			256 => Some(Bitrate::Rate256),
			288 => Some(Bitrate::Rate288),
			320 => Some(Bitrate::Rate320),
			352 => Some(Bitrate::Rate352),
			384 => Some(Bitrate::Rate384),
			416 => Some(Bitrate::Rate416),
			448 => Some(Bitrate::Rate448),
			_ => None,
		}
	}

	pub const fn kbps(&self) -> Option<usize> {
		match self {
			Bitrate::RateFree => None,
			Bitrate::Rate8 => Some(8),
			Bitrate::Rate16 => Some(16),
			Bitrate::Rate24 => Some(24),
			Bitrate::Rate32 => Some(32),
			Bitrate::Rate40 => Some(40),
			Bitrate::Rate48 => Some(48),
			Bitrate::Rate56 => Some(56),
			Bitrate::Rate64 => Some(64),
			Bitrate::Rate80 => Some(80),
			Bitrate::Rate96 => Some(96),
			Bitrate::Rate112 => Some(112),
			Bitrate::Rate128 => Some(128),
			Bitrate::Rate144 => Some(144),
			Bitrate::Rate160 => Some(160),
			Bitrate::Rate176 => Some(176),
			Bitrate::Rate192 => Some(192),
			Bitrate::Rate224 => Some(224),
			Bitrate::Rate256 => Some(256),
			Bitrate::Rate288 => Some(288),
			Bitrate::Rate320 => Some(320),
			Bitrate::Rate352 => Some(352),
			Bitrate::Rate384 => Some(384),
			Bitrate::Rate416 => Some(416),
			Bitrate::Rate448 => Some(448),
		}
	}

	pub const fn bitrate(&self) -> Option<usize> {
		match self.kbps() {
			None => None,
			Some(kbr) => Some(kbr * 1000),
		}
	}
}

pub enum Fourbit {
	Zero,
	One,
	Two,
	Three,
	Four,
	Five,
	Six,
	Seven,
	Eight,
	Nine,
	Ten,
	Eleven,
	Twelve,
	Thirteen,
	Fourteen,
	Fifteen,
}

impl Fourbit {
	pub fn from_u8(value: u8) -> Option<Self> {
		if value > 15 {
			return None;
		}

		Some(match value {
			0 => Fourbit::Zero,
			1 => Fourbit::One,
			2 => Fourbit::Two,
			3 => Fourbit::Three,
			4 => Fourbit::Four,
			5 => Fourbit::Five,
			6 => Fourbit::Six,
			7 => Fourbit::Seven,
			8 => Fourbit::Eight,
			9 => Fourbit::Nine,
			10 => Fourbit::Ten,
			11 => Fourbit::Eleven,
			12 => Fourbit::Twelve,
			13 => Fourbit::Thirteen,
			14 => Fourbit::Fourteen,
			15 => Fourbit::Fifteen,
			_ => unreachable!(),
		})
	}
}

#[cfg(test)]
mod test {

	use super::{Bitrate, Layer, Version};

	// Lookup table of bitrates excluding 0000 and 1111. The former is Free, the latter is Bad
	#[rustfmt::skip]
	const BR_LUT: [usize; 5 * 14] = [
		32,  32,  32,  32,  8,
		64,  48,  40,  48,  16,
		96,  56,  48,  56,  24,
		128, 64,  56,  64,  32,
		160, 80,  64,  80,  40,
		192, 96,  80,  96,  48,
		224, 112, 96,  112, 56,
		256, 128, 112, 128, 64,
		288, 160, 128, 144, 80,
		320, 192, 160, 160, 96,
		352, 224, 192, 176, 112,
		384, 256, 224, 192, 128,
		416, 320, 256, 224, 144,
		448, 384, 320, 256, 160
	];

	fn bitrate_lut() -> Vec<Bitrate> {
		BR_LUT
			.into_iter()
			.map(|kbps| Bitrate::from_kbps(kbps).unwrap())
			.collect()
	}

	#[test]
	fn correctly_resolves_bitrates() {
		let lut = bitrate_lut();
		for index in 0..75 {
			let br = (index as u8 / 5) << 4;

			let (version, layer) = match index % 5 {
				0 => (Version::Mpeg1, Layer::Layer1),
				1 => (Version::Mpeg1, Layer::Layer2),
				2 => (Version::Mpeg1, Layer::Layer3),
				3 => (Version::Mpeg2, Layer::Layer1),
				4 => (Version::Mpeg2, Layer::Layer2),
				_ => unreachable!(),
			};

			let resolved_bitrate = Bitrate::resolve(br, version, layer).unwrap();
			let bre = match index {
				0 | 1 | 2 | 3 | 4 => Bitrate::RateFree,
				_ => lut[index - 5],
			};

			if resolved_bitrate != bre {
				panic!("Failed on {:04b}, {version:?}, {layer:?}", br >> 4);
			}

			assert_eq!(bre, resolved_bitrate)
		}
	}
}
