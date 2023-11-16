use std::collections::HashMap;

use bitvec::prelude::*;

pub struct LZW {
	minimum_size: u8,
	clear_code: u16,
	end_of_information_code: u16,
	dictionary: HashMap<Vec<u8>, u16>,
}
impl LZW {
	pub fn new(minimum_size: u8) -> Self {
		let mut dictionary: HashMap<Vec<u8>, u16> = HashMap::new();

		let clear_code = 1 << minimum_size;
		let end_of_information_code = clear_code + 1;

		println!("mcs {minimum_size} | cc {clear_code}");

		// Fill dictionary with self-descriptive values
		for value in 0..clear_code {
			dictionary.insert(vec![value as u8], value);
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

	pub fn encode(&mut self, indices: &[u8]) -> Vec<u8> {
		let mut next_code = self.end_of_information_code + 1;
		let mut code_size = self.minimum_size + 1;

		let mut iter = indices.iter();
		let mut out = BitStream::new();
		let mut buffer = vec![*iter.next().unwrap()];

		out.push_bits(code_size, self.clear_code);

		for &index in iter {
			buffer.push(index);

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

					buffer = vec![index];
				} else {
					println!("index is: {index}");
					println!("buffer is: {:?}", buffer);
					println!("dictionary: {:?}", self.dictionary);
					unreachable!()
				}
			}
		}

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

	fn rand_against_weezl(length: usize) {
		let range = rand::distributions::Uniform::from(0..=1);
		let indices = rand::Rng::sample_iter(rand::thread_rng(), &range)
			.take(length)
			.collect::<Vec<_>>();
		let weezl = weezl::encode::Encoder::new(weezl::BitOrder::Lsb, 2)
			.encode(&indices)
			.unwrap();
		let us = LZW::new(2).encode(&indices);

		assert_eq!(us.len(), weezl.len());
	}

	#[test]
	#[ignore]
	fn fortyk_against_weezl() {
		rand_against_weezl(40_000);
	}

	#[test]
	#[ignore]
	fn thirtyeightk_against_weezl() {
		rand_against_weezl(38_000);
	}

	#[test]
	#[ignore]
	fn twentyk_against_weezl_repeated() {
		for _ in 0..100 {
			rand_against_weezl(20_000)
		}
	}

	#[test]
	fn encode() {
		let indices = vec![0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0];
		let output = vec![0x84, 0x1D, 0x81, 0x7A, 0x50];

		let lzout = LZW::new(2).encode(&indices);

		assert_eq!(lzout, output);
	}

	#[test]
	fn against_weezl() {
		let indices = vec![0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0];
		let weezl = weezl::encode::Encoder::new(weezl::BitOrder::Lsb, 2)
			.encode(&indices)
			.unwrap();
		let us = LZW::new(2).encode(&indices);

		assert_eq!(weezl, us);
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
