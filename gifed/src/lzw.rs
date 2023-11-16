use std::collections::HashMap;

use bitvec::prelude::*;

#[rustfmt::skip]
const DEFAULT_DICT: [u8; 256] = [
	0,   1,   2,   3,   4,   5,   6,   7,   8,   9,  10,  11,  12,  13,  14,  15, 
	16,  17,  18,  19,  20,  21,  22,  23,  24,  25,  26,  27,  28,  29,  30,  31, 
	32,  33,  34,  35,  36,  37,  38,  39,  40,  41,  42,  43,  44,  45,  46,  47, 
	48,  49,  50,  51,  52,  53,  54,  55,  56,  57,  58,  59,  60,  61,  62,  63, 
	64,  65,  66,  67,  68,  69,  70,  71,  72,  73,  74,  75,  76,  77,  78,  79, 
	80,  81,  82,  83,  84,  85,  86,  87,  88,  89,  90,  91,  92,  93,  94,  95, 
	96,  97,  98,  99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 
	112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 
	128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 
	144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 
	160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 174, 175, 
	176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191, 
	192, 193, 194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206, 207, 
	208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223, 
	224, 225, 226, 227, 228, 229, 230, 231, 232, 233, 234, 235, 236, 237, 238, 239, 
	240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255, 
];

pub struct LZW<'a> {
	minimum_size: u8,
	clear_code: u16,
	end_of_information_code: u16,
	dictionary: HashMap<&'a [u8], u16>,
}

impl<'a> LZW<'a> {
	pub fn new(minimum_size: u8) -> Self {
		let mut dictionary: HashMap<&'a [u8], u16> = HashMap::new();

		let clear_code = 1u16 << minimum_size;
		let end_of_information_code = clear_code + 1;

		dbg!(minimum_size, clear_code);

		// Fill dictionary with self-descriptive values
		for value in 0..clear_code {
			dictionary.insert(&DEFAULT_DICT[value as usize..value as usize + 1], value);
		}

		Self {
			minimum_size,
			clear_code,
			end_of_information_code,
			dictionary,
		}
	}

	pub fn reset(&mut self) {
		*self = Self::new(self.minimum_size)
	}

	pub fn decode(&mut self, encoded: &[u8]) -> Vec<u8> {
		let mut input = BitStream::new();
		for &byte in encoded {
			input.push_bits(8, byte as u16);
		}

		let mut out = BitStream::new();

		todo!();

		out.vec()
	}

	pub fn encode(&mut self, indices: &'a [u8]) -> Vec<u8> {
		let mut next_code = self.end_of_information_code + 1;
		let mut code_size = self.minimum_size + 1;

		let mut out = BitStream::new();
		let mut buffer_start = 0;
		let mut buffer_len = 0;

		out.push_bits(code_size, self.clear_code);

		for idx in 0..indices.len() {
			buffer_len += 1;
			let buffer = &indices[buffer_start..buffer_start + buffer_len];

			if !self.dictionary.contains_key(&buffer) {
				if let Some(&code) = self.dictionary.get(&buffer[..buffer.len() - 1]) {
					out.push_bits(code_size, code);

					// add the vec to the dict
					self.dictionary.insert(buffer, next_code);
					next_code += 1;

					// If the next_code can't fit in the code_size, we have to increase it
					if next_code - 1 == 1 << code_size {
						code_size += 1;
					}

					if next_code >= (1 << 12) {
						out.push_bits(code_size, self.clear_code);
						self.reset();
						next_code = self.end_of_information_code + 1;
						code_size = self.minimum_size + 1;
					}

					buffer_start = idx;
					buffer_len = 1;
				} else {
					println!("index is: {idx}");
					println!("buffer is: {:?}", buffer);
					println!("dictionary: {:?}", self.dictionary);
					unreachable!()
				}
			}
		}

		let buffer = &indices[buffer_start..buffer_start + buffer_len];
		if !buffer.is_empty() {
			match self.dictionary.get(&buffer) {
				Some(&code) => out.push_bits(code_size, code),
				None => {
					unreachable!(
						"Codes left in the buffer but the buffer is not a valid dictionary key!"
					)
				}
			}
		}
		out.push_bits(code_size, self.end_of_information_code);

		out.vec()
	}
}

#[cfg(test)]
mod lzw_test {
	use super::*;

	fn rand_against_weezl(length: usize, lzw_size: u8) {
		let range = rand::distributions::Uniform::from(0..=((1 << lzw_size as usize) - 1) as u8);
		let indices = rand::Rng::sample_iter(rand::thread_rng(), &range)
			.take(length)
			.collect::<Vec<_>>();
		let weezl = weezl::encode::Encoder::new(weezl::BitOrder::Lsb, lzw_size)
			.encode(&indices)
			.unwrap();
		let us = LZW::new(lzw_size).encode(&indices);

		let weezl_decode = weezl::decode::Decoder::new(weezl::BitOrder::Lsb, lzw_size)
			.decode(&weezl)
			.unwrap();

		let us_decode = weezl::decode::Decoder::new(weezl::BitOrder::Lsb, lzw_size)
			.decode(&us)
			.unwrap();

		assert_eq!(us_decode, weezl_decode);
	}

	#[test]
	fn fortyk_against_weezl() {
		rand_against_weezl(40_000, 2);
	}

	#[test]
	fn thirtyeightk_against_weezl() {
		rand_against_weezl(38_000, 2);
	}

	#[test]
	fn thirtyeightk_against_weezl_full_codesize() {
		rand_against_weezl(38_000, 8);
	}

	#[test]
	fn twentyk_against_weezl_repeated() {
		for _ in 0..100 {
			rand_against_weezl(20_000, 2)
		}
	}

	#[test]
	fn encode() {
		let indices = vec![0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0];
		let output = vec![0x84, 0x1D, 0x81, 0x7A, 0x50];

		let lzout = LZW::new(2).encode(&indices);

		assert_eq!(lzout, output);
	}
}

struct BitStream {
	formed: BitVec<u8, Lsb0>,
}

impl BitStream {
	fn new() -> Self {
		Self {
			formed: BitVec::EMPTY,
		}
	}

	fn push_bits(&mut self, count: u8, data: u16) {
		for i in 0..count {
			self.formed.push((data & (1 << i)) > 0)
		}
	}

	fn pop_bits(&mut self, count: u8) -> u16 {
		let mut out = 0;
		for i in (0..count).filter_map(|_| self.formed.pop()) {
			out <<= 1;
			let int: u16 = i.into();
			out |= int;
		}
		out
	}

	fn vec(mut self) -> Vec<u8> {
		self.formed.set_uninitialized(false);
		self.formed.into_vec()
	}
}

#[cfg(test)]
mod bitstream_test {
	use super::*;

	#[test]
	fn short_push() {
		let mut bs = BitStream::new();
		bs.push_bits(2, 3);
		bs.push_bits(2, 3);
		bs.push_bits(3, 1);
		bs.push_bits(2, 3);

		let bsvec = bs.vec();

		for byte in &bsvec {
			print!("{:b} ", byte);
		}
		println!();

		assert_eq!(bsvec, vec![0b1001_1111, 0b0000_0001]);
	}

	#[test]
	fn long_push() {
		let mut bs = BitStream::new();
		bs.push_bits(1, 1);
		bs.push_bits(12, 2049);

		let bsvec = bs.vec();

		for byte in &bsvec {
			print!("{:b} ", byte);
		}
		println!();

		assert_eq!(bsvec, vec![0b0000_0011, 0b0001_0000]);
	}
}
