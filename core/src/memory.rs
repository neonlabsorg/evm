use core::cmp::{min, max};
use alloc::{vec,vec::Vec};
use crate::{ExitError, ExitFatal};

/// A sequencial memory. It uses Rust's `Vec` for internal
/// representation.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(borsh::BorshSerialize, borsh::BorshDeserialize)]
pub struct Memory {
	#[cfg_attr(feature = "with-serde", serde(with = "serde_bytes"))]
	data: Vec<u8>,
	effective_len: usize,
	limit: usize,
}

impl Memory {
	/// Create a new memory with the given limit.
	#[must_use]
	pub const fn new(limit: usize) -> Self {
		Self {
			data: Vec::new(),
			effective_len: 0_usize,
			limit,
		}
	}

	/// Memory limit.
	#[must_use]
	pub const fn limit(&self) -> usize {
		self.limit
	}

	/// Get the length of the current memory range.
	#[must_use]
	pub fn len(&self) -> usize {
		self.data.len()
	}

	/// Get the effective length.
	#[must_use]
	pub const fn effective_len(&self) -> usize {
		self.effective_len
	}

	/// Return true if current effective memory range is zero.
	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	pub fn data(&self) -> &[u8] {
		&self.data
	}

	/// Resize the memory, making it cover the memory region of `offset..(offset
	/// + len)`, with 32 bytes as the step. If the length is zero, this function
	/// does nothing.
	pub fn resize_offset(&mut self, offset: usize, len: usize) -> Result<(), ExitError> {
		if len == 0 {
			return Ok(())
		}

		offset.checked_add(len).map_or(Err(ExitError::InvalidRange), |end| self.resize_end(end))
	}

	/// Resize the memory, making it cover to `end`, with 32 bytes as the step.
	pub fn resize_end(&mut self, end: usize) -> Result<(), ExitError> {
		let end = {
			let modulo = end % 32;
			if modulo == 0 {
				end
			} else {
				// next closest value to `end` that is divisible by 32
				// end = (end + 32) - (end % 32)
				match end.checked_add(32) {
					Some(end) => end - modulo,
					None => return Err(ExitError::InvalidRange)
				}
			}
		};

		self.effective_len = max(self.effective_len, end);
		Ok(())
	}

	/// Get memory region at given offset.
	///
	/// ## Panics
	///
	/// Value of `size` is considered trusted. If they're too large,
	/// the program can run out of memory, or it can overflow.
	#[must_use]
	pub fn get(&self, offset: usize, size: usize) -> Vec<u8> {
		let mut ret = vec![0; size];

		if offset >= self.data.len() {
			return ret;
		}
		let end = match offset.checked_add(size) {
			Some(end) => min(end, self.data.len()),
			None => return ret
		};

		(&mut ret[0..(end - offset)]).copy_from_slice(&self.data[offset..end]);

		ret
	}

	/// Set memory region at given offset. The offset and value is considered
	/// untrusted.
	pub fn set(
		&mut self,
		offset: usize,
		value: &[u8],
		target_size: Option<usize>
	) -> Result<(), ExitFatal> {
		let target_size = target_size.unwrap_or(value.len());

		if offset.checked_add(target_size).map_or(true, |pos| pos > self.limit)
		{
			return Err(ExitFatal::NotSupported)
		}

		if self.data.len() < offset + target_size {
			self.data.resize(offset + target_size, 0);
		}

		let data = &mut self.data[offset..(offset + target_size)];
		let value_size = min(value.len(), target_size);
		let (d1, d2) = data.split_at_mut(value_size);
		d1.copy_from_slice(&value[0..value_size]);
		d2.fill(0);

		Ok(())
	}

	/// Copy `data` into the memory, of given `len`.
	pub fn copy_large(
		&mut self,
		memory_offset: usize,
		data_offset: usize,
		len: usize,
		data: &[u8]
	) -> Result<(), ExitFatal> {
		let data_by_offset = data_offset.checked_add(len).map_or(&[][..], |end| {
			if data_offset > data.len() {
				&[][..]
			} else {
				&data[data_offset..min(end, data.len())]
			}
		});

		self.set(memory_offset, data_by_offset, Some(len))
	}
}
