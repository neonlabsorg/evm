use alloc::{vec, vec::Vec};

/// Mapping of valid jump destination from code.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(borsh::BorshSerialize, borsh::BorshDeserialize)]
pub struct Valids{
	#[cfg_attr(feature = "with-serde", serde(with = "serde_bytes"))]
	data: Vec<u8>
}

impl Valids {
	/// Create a new valid mapping from given code bytes.
	#[must_use]
	pub fn new(valids: Vec<u8>) -> Self {
		Self{ data: valids }
	}

	/// Returns `true` if the position is a valid jump destination. If
	/// not, returns `false`.
	#[must_use]
	pub fn is_valid(&self, position: usize) -> bool {
		if position >= (self.data.len() * 8) {
			return false;
		}

		let byte_index = position / 8;
		let byte = self.data[byte_index];

		let bit_index = position % 8;
		let bit_test = 1_u8 << bit_index;

		(byte & bit_test) == bit_test
	}

	#[must_use]
	pub fn compute(code: &[u8]) -> Vec<u8> {
		let mut valids: Vec<u8> = vec![0; Self::size_needed(code.len())];
	
		let mut i = 0;
		while i < code.len() {
			let opcode = code[i];
			match opcode {
				0x5b => { // Jump Dest
					let byte: &mut u8 = &mut valids[i / 8];
					*byte |= 1_u8 << (i % 8);
				},
				0x60..=0x7f => { // Push
					i += (opcode as usize) - 0x60 + 1;
				},
				_ => {}
			}
	
			i += 1;
		}
	
		valids
	}

	#[inline]
	#[must_use]
	/// Returns minimal number of bytes needed for storing `valids` bitmap.
	pub fn size_needed(code_len: usize) -> usize {
		(code_len + 7) >> 3
	}
}

#[cfg(test)]
mod tests {
	use crate::Valids;

	#[test]
	fn test_size_needed() {
		assert_eq!(Valids::size_needed(0), 0);
		assert_eq!(Valids::size_needed(1), 1);
		assert_eq!(Valids::size_needed(8), 1);
		assert_eq!(Valids::size_needed(9), 2);
		assert_eq!(Valids::size_needed(16), 2);
		assert_eq!(Valids::size_needed(17), 3);
	}
}
