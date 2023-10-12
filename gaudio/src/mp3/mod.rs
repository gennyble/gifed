use std::io::{BufRead, BufReader, Cursor, ErrorKind, Read};

use crate::mp3::bitrate::Bitrate;

mod bitrate;

/// Destroy an MP3, ripping it's frames apart. Also removes any ID3v2 tags
/// because who needs metadata?
pub struct Breaker {
	pub frames: Vec<Frame>,
}

impl Breaker {
	pub fn new() -> Self {
		Self { frames: vec![] }
	}

	pub fn split(&mut self, data: Vec<u8>) -> Result<(), std::io::Error> {
		let cursor = Cursor::new(data);
		let mut reader = BufReader::new(cursor);

		let mut consumed = 0;
		loop {
			print!("[{consumed:06X}] reading... ");
			let mut three = [0x00, 0x00, 0x00];
			if let Err(e) = reader.read_exact(&mut three) {
				if e.kind() == ErrorKind::UnexpectedEof {
					println!("out of bytes!");
					break;
				} else {
					println!("failed!");
					return Err(e);
				}
			}
			consumed += 3;

			if &three == b"ID3" {
				println!("found ID3v2!");
				Self::skip_id3v2(&mut reader, &mut consumed)?
			} else if three[0] == 0xFF && three[1] & 0b1110_0000 == 0b1110_0000 {
				print!("Have header - ");
				let mut one_more = [0x00];
				reader.read_exact(&mut one_more)?;
				consumed += 1;

				let header =
					Header::from_bytes([three[0], three[1], three[2], one_more[0]]).unwrap();
				let dat_len = header.data_length();
				let mut data = vec![0; dat_len];
				reader.read_exact(&mut data)?;
				consumed += dat_len;

				println!(
					"{}kbps {}kHz {}bytes",
					header.bitrate.kbps().unwrap(),
					header.samplerate.freq() / 1000,
					header.length()
				);

				self.frames.push(Frame { header, data });
			} else {
				println!("unsynced!");
				panic!()
			}
		}

		Ok(())
	}

	/// Assumes the ident "TAG" was already consumed
	fn skip_id3v2<R: BufRead>(reader: &mut R, consumed: &mut usize) -> Result<(), std::io::Error> {
		// We don't actually want this, but want to get rid of it.
		let mut version_and_flags = [0x00, 0x00, 0x00];
		reader.read_exact(&mut version_and_flags)?;
		*consumed += 3;

		println!(
			"Version {} Revision {}",
			version_and_flags[0], version_and_flags[1]
		);

		let mut syncsafe_size = [0x00, 0x00, 0x00, 0x00];
		reader.read_exact(&mut syncsafe_size)?;
		*consumed += 4;

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
		reader.read_exact(&mut skip)?;
		*consumed += size as usize;

		Ok(())
	}
}

pub struct Frame {
	pub header: Header,
	pub data: Vec<u8>,
}

pub struct Header {
	// I only want to parse what i need, but we need this for writing out, so
	pub raw: [u8; 4],
	pub version: Version,
	pub layer: Layer,
	pub crc: bool,
	pub bitrate: Bitrate,
	pub samplerate: SampleRate,
	pub pad: bool,
}

impl Header {
	pub fn from_bytes(raw: [u8; 4]) -> Result<Self, Error> {
		if raw[0] != 0xFF || raw[1] & 0b1110_0000 != 0b1110_0000 {
			return Err(Error::HeaderUnsync);
		}

		//TODO: gen- yell if the version and layer aren't V1 L3?
		let version = Version::from_packed(raw[1]);
		let layer = Layer::from_packed(raw[1]);
		// CRC is 2bytes and directly follows the frame header
		let crc = raw[1] & 1 == 0;
		let bitrate = Bitrate::resolve(raw[2], version, layer)?;
		let samplerate = SampleRate::from_packed(raw[2]);

		if let SampleRate::Reserved = samplerate {
			return Err(Error::SampleRateReserve);
		}

		let pad = raw[2] & 2 > 0;

		//TODO: gen- love, you were trying to get the size of the data field. We need
		//to know the sampling rate and the pad bit for that, which happen to be the
		//next three bits.

		//Things i did not parse because i do not care about them:
		// - private bit
		// - channels
		// - mode extension
		// - copyright (lol)
		// - original (lmfao)
		// - emphasis

		Ok(Self {
			raw,
			version,
			layer,
			crc,
			bitrate,
			samplerate,
			pad,
		})
	}

	// Algorithm taken from:
	// http://www.multiweb.cz/twoinches/mp3inside.htm
	/// The length of the header and data
	pub fn length(&self) -> usize {
		// what, do we not care about crc? won't it add 2 bytes?
		let size = (144 * self.bitrate.bitrate().unwrap()) / self.samplerate.freq();
		if self.pad {
			size + 1
		} else {
			size
		}
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
	BitrateReserve,
	#[error("Bitrate bits were all 1")]
	BitrateBad,
	#[error("SampleRate was a reserved value")]
	SampleRateReserve,
}

#[derive(Copy, Clone, Debug)]
pub enum Version {
	Mpeg2_5,
	Reserved,
	Mpeg2,
	Mpeg1,
}

impl Version {
	/// Parse the Version from the second byte of the frame header
	fn from_packed(byte: u8) -> Self {
		#[allow(clippy::unusual_byte_groupings)]
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

impl Layer {
	/// Parse the Layer from the second byte of the frame header.
	fn from_packed(byte: u8) -> Self {
		#[allow(clippy::unusual_byte_groupings)]
		match byte & 0b000_00_110 {
			0b000_00_000 => Layer::Reserved,
			0b000_00_010 => Layer::Layer3,
			0b000_00_100 => Layer::Layer2,
			0b000_00_110 => Layer::Layer1,
			_ => unreachable!(),
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub enum SampleRate {
	Hz44100,
	Hz48000,
	Hz32000,
	Reserved,
}

impl SampleRate {
	/// Parse the SampleRate from the third byte of the frame header
	fn from_packed(byte: u8) -> Self {
		#[allow(clippy::unusual_byte_groupings)]
		match byte & 0b0000_11_0_0 {
			0b0000_00_0_0 => SampleRate::Hz44100,
			0b0000_01_0_0 => SampleRate::Hz48000,
			0b0000_10_0_0 => SampleRate::Hz32000,
			0b0000_11_0_0 => SampleRate::Reserved,
			_ => unreachable!(),
		}
	}

	pub fn freq(&self) -> usize {
		match self {
			SampleRate::Hz44100 => 44100,
			SampleRate::Hz48000 => 48000,
			SampleRate::Hz32000 => 32000,
			SampleRate::Reserved => {
				panic!("sample rate was a reserved value; unable to determien a frequency")
			}
		}
	}
}
