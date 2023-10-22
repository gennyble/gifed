use std::collections::HashMap;

pub struct LZW {}
impl LZW {
	pub fn encode(minimum_size: u8, indices: &[u8]) -> Vec<u8> {
		let mut dictionary: HashMap<Vec<u8>, u16> = HashMap::new();

		let cc = 1 << minimum_size;
		let eoi = cc + 1;

		println!("mcs {minimum_size} | cc {cc}");

		// Fill dictionary with self-descriptive values
		for value in 0..cc {
			dictionary.insert(vec![value as u8], value);
		}

		let mut next_code = eoi + 1;
		let mut code_size = minimum_size + 1;

		let mut iter = indices.iter();
		let mut out = BitStream::new();
		let mut buffer = vec![*iter.next().unwrap()];

		out.push_bits(code_size, cc);

		for &index in iter {
			buffer.push(index);

			if !dictionary.contains_key(&buffer) {
				buffer.pop();

				if let Some(&code) = dictionary.get(&buffer) {
					out.push_bits(code_size, code);

					// Put the code back and add the vec to the dict
					buffer.push(indicie);
					dictionary.insert(buffer.clone(), next_code);
					next_code += 1;

					// If the next_code can't fit in the code_size, we have to increase it
					if next_code - 1 == 1 << code_size {
						code_size += 1;
					}

					buffer.clear();
					buffer.push(indicie);
				} else {
					println!("indicie is: {}", indicie);
					println!("buffer is: {:?}", buffer);
					println!("dictionary: {:?}", dictionary);
					unreachable!()
				}
			}
		}

		if !buffer.is_empty() {
			match dictionary.get(&buffer) {
				Some(&code) => out.push_bits(code_size, code),
				None => {
					unreachable!(
						"Codes left in the buffer but the buffer is not a valid dictionary key!"
					)
				}
			}
		}
		out.push_bits(code_size, eoi);

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
		let us = LZW::encode(2, &indices);

		assert_eq!(us.len(), weezl.len());
	}

	#[test]
	fn fortyk_against_weezl() {
		rand_against_weezl(40_000);
	}

	#[test]
	#[ignore]
	fn thirtyeightk_against_weezl() {
		rand_against_weezl(38_000);
	}

	#[test]
	fn encode() {
		let indices = vec![0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0];
		let output = vec![0x84, 0x1D, 0x81, 0x7A, 0x50];

		let lzout = LZW::encode(2, &indices);

		assert_eq!(lzout, output);
	}

	#[test]
	fn against_weezl() {
		let indices = vec![0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0];
		let weezl = weezl::encode::Encoder::new(weezl::BitOrder::Lsb, 2)
			.encode(&indices)
			.unwrap();
		let us = LZW::encode(2, &indices);

		assert_eq!(weezl, us);
	}
}

struct BitStream {
	formed: Vec<u8>,
	current: u8,
	index: u8,
}

impl BitStream {
	fn new() -> Self {
		Self {
			formed: vec![],
			current: 0,
			index: 0,
		}
	}

	fn push_bits(&mut self, count: u8, data: u16) {
		let mut new_index = self.index + count;
		let mut current32 = (self.current as u32) | ((data as u32) << self.index);

		loop {
			if new_index >= 8 {
				self.formed.push(current32 as u8);
				current32 >>= 8;
				new_index -= 8;
			} else {
				self.current = current32 as u8;
				self.index = new_index;

				break;
			}
		}
	}

	fn vec(self) -> Vec<u8> {
		let mut out = self.formed;

		if self.index != 0 {
			out.push(self.current);
		}

		out
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
