use alloc::{vec, vec::Vec};

/// Mapping of valid jump destination from code.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
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
		let valids_bytes_len = (code.len() / 8) + 1;
		let mut valids: Vec<u8> = vec![0; valids_bytes_len];
	
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
}
