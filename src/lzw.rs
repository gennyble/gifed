use std::collections::HashMap;

pub struct LZW {}
impl LZW {
	//TODO: Cleanup and make not awful
	pub fn encode(minimum_size: u8, indicies: &[u8]) -> Vec<u8> {
		let mut dictionary: HashMap<Vec<u8>, u16> = HashMap::new();

		let cc: u16 = 2u16.pow(minimum_size as u32);
		let eoi: u16 = cc+1;

		let mut index: u16 = eoi+1; // Next code number
		let mut codesize: u8 = minimum_size+1; // Current code size

		//println!("cc: {} - eoi: {}", cc, eoi);

		// Populate starting codes
		for code in 0..cc {
			dictionary.insert(vec![code as u8], code);
		}

		let mut iter = indicies.iter();
		let mut out = BitStream::new();
		let mut buffer = vec![*iter.next().unwrap()]; //TODO: Remove unwrap

		//"Encoders should output a Clear code as the first code of each image data stream."
		out.push_bits(codesize, cc);

		//println!("Before Loop\n\tBuffer: {:?}\n\tCodesize:{}\n\tIndex:{}", buffer, codesize, index);

		for next_code in iter {
			buffer.push(*next_code);

			//println!("Buffer: {:?} - Codesize:{} - Index:{}\n\tDict: {:?}", buffer, codesize, index, dictionary);

			match dictionary.get(&buffer) {
				Some(_code) => {
					//println!("\tPresent!");
					continue;
				},
				None => {
					buffer.pop();
					match dictionary.get(&buffer) {
						Some(code) => {
							out.push_bits(codesize, *code);
							//println!("\tOutputting {} with {} bits. Buffer is {:?}", *code, codesize, buffer);

							// Add new entry for buffer and increase the index
							buffer.push(*next_code);
							dictionary.insert(buffer, index);
							index += 1;

							// Increase code size if we should
							if index-1 == 2u16.pow(codesize as u32) {
								codesize += 1;
							}

							// Reset the buffer to only contain next_code
							buffer = vec![*next_code];
						},
						None => panic!("No dictionary entry when there should be! Something is wrong!")
					}
				}
			}
		}

		if buffer.len() > 0 {
			match dictionary.get(&buffer) {
				None => panic!("No codes left but not in the dictionary!"),
				Some(code) => out.push_bits(codesize, *code)
			}
		}
		out.push_bits(codesize, eoi);

		out.vec()
	}
}

#[cfg(test)]
mod lzw_test {
	use super::*;

	#[test]
	fn encode() {
		let indicies = vec![0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0];
		let output = vec![0x84, 0x1D, 0x81, 0x7A, 0x50];

		let lzout = LZW::encode(2, &indicies);

		assert_eq!(lzout, output);
	}
}

struct BitStream {
	formed: Vec<u8>,
	current: u8,
	index: u8
}

impl BitStream {
	fn new() -> Self {
		Self {
			formed: vec![],
			current: 0,
			index: 0
		}
	}

	fn push_bits(&mut self, count: u8, data: u16) {
		let mut new_index = self.index + count;
		let mut current32 = (self.current as u32) | ((data as u32) << self.index);

		loop {
			if new_index >= 8 {
				self.formed.push(current32 as u8);
				current32 = current32 >> 8;
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
		println!("");

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
		println!("");

		assert_eq!(bsvec, vec![0b0000_0011, 0b0001_0000]);
	}
}