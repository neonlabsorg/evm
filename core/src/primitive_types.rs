#![allow(clippy::use_self)]

use fixed_hash::{construct_fixed_hash, impl_fixed_hash_conversions};
use uint::{construct_uint};


construct_fixed_hash! {
	/// Fixed-size uninterpreted hash type with 20 bytes (160 bits) size.
	pub struct H160(20);
}
construct_fixed_hash! {
	/// Fixed-size uninterpreted hash type with 32 bytes (256 bits) size.
	pub struct H256(32);
}

impl_fixed_hash_conversions!(H256, H160);


construct_uint! {
	/// 256-bit unsigned integer.
	pub struct U256(4);
}

construct_uint! {
	/// 512-bit unsigned integer.
	pub struct U512(8);
}

impl_rlp::impl_uint_rlp!(U256, 4);
impl_rlp::impl_fixed_hash_rlp!(H160, 20);
impl_rlp::impl_fixed_hash_rlp!(H256, 32);

impl_serde::impl_uint_serde!(U256, 4);
impl_serde::impl_fixed_hash_serde!(H160, 20);
impl_serde::impl_fixed_hash_serde!(H256, 32);


impl From<U256> for U512 {
	fn from(value: U256) -> U512 {
		let U256(ref arr) = value;
		let mut ret = [0; 8];
		ret[0] = arr[0];
		ret[1] = arr[1];
		ret[2] = arr[2];
		ret[3] = arr[3];
		U512(ret)
	}
}

/// Error type for conversion.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
	/// Overflow encountered.
	Overflow,
}

impl core::convert::TryFrom<U512> for U256 {
	type Error = Error;

	fn try_from(value: U512) -> Result<U256, Error> {
		let U512(ref arr) = value;
		if arr[4] | arr[5] | arr[6] | arr[7] != 0 {
			return Err(Error::Overflow);
		}
		let mut ret = [0; 4];
		ret[0] = arr[0];
		ret[1] = arr[1];
		ret[2] = arr[2];
		ret[3] = arr[3];
		Ok(U256(ret))
	}
}


/// Add Serde serialization support to a fixed-sized hash type created by `construct_fixed_hash!`.
#[macro_export]
macro_rules! impl_fixed_hash_borsh {
	($name: ident) => {
		impl borsh::BorshSerialize for $name {
			fn serialize<W: borsh::maybestd::io::Write>(&self, writer: &mut W) -> borsh::maybestd::io::Result<()> {
				writer.write_all(&self.0)?;

				Ok(())
			}
		
			fn u8_slice(slice: &[Self]) -> Option<&[u8]>
			{
				let (_, data, _) = unsafe { slice.align_to::<u8>() };
				Some(data)
			}
		}
		
		impl borsh::BorshDeserialize for $name {
			fn deserialize(buf: &mut &[u8]) -> borsh::maybestd::io::Result<Self> {
				let bytes_len = core::mem::size_of::<$name>();

				if buf.len() < bytes_len {
					return Err(borsh::maybestd::io::Error::new(
						borsh::maybestd::io::ErrorKind::InvalidInput,
						"ERROR_UNEXPECTED_LENGTH_OF_INPUT"
					));
				}

				let (front, rest) = buf.split_at(bytes_len);
				*buf = rest;

				Ok(Self::from_slice(front))
			}
		
			fn vec_from_bytes(len: u32, buf: &mut &[u8]) -> borsh::maybestd::io::Result<Option<borsh::maybestd::vec::Vec<Self>>> {
				let bytes_len = core::mem::size_of::<$name>() * (len as usize);
		
				let mut vec = borsh::maybestd::vec::Vec::with_capacity(len as usize);
		
				let (front, rest) = buf.split_at(bytes_len);
				*buf = rest;
		
				unsafe {
					let ptr = vec.as_mut_ptr() as *mut u8;
					ptr.copy_from_nonoverlapping(front.as_ptr(), bytes_len);
		
					vec.set_len(len as usize);
				}
		
				Ok(Some(vec))
			}
		
			fn copy_from_bytes(buf: &mut &[u8], out: &mut [Self]) -> borsh::maybestd::io::Result<bool> {
				let bytes_len = core::mem::size_of::<$name>() * out.len();
		
				let (front, rest) = buf.split_at(bytes_len);
				*buf = rest;
		
				let (_, out, _) = unsafe { out.align_to_mut::<u8>() };
				out.copy_from_slice(front);
		
				Ok(true)
			}
		}
	};
}

impl_fixed_hash_borsh!(H160);
impl_fixed_hash_borsh!(H256);


impl borsh::BorshSerialize for U256 {
	fn serialize<W: borsh::maybestd::io::Write>(&self, writer: &mut W) -> borsh::maybestd::io::Result<()> {
		let buffer: [u8; 32] = unsafe { core::mem::transmute_copy(self) };
		writer.write_all(&buffer)?;

		Ok(())
	}

	fn u8_slice(slice: &[Self]) -> Option<&[u8]>
	{
		let (_, data, _) = unsafe { slice.align_to::<u8>() };
		Some(data)
	}
}

impl borsh::BorshDeserialize for U256 {
	fn deserialize(buf: &mut &[u8]) -> borsh::maybestd::io::Result<Self> {
		let bytes_len = core::mem::size_of::<U256>();

		if buf.len() < bytes_len {
			return Err(borsh::maybestd::io::Error::new(
				borsh::maybestd::io::ErrorKind::InvalidInput,
				"ERROR_UNEXPECTED_LENGTH_OF_INPUT"
			));
		}

		let (front, rest) = buf.split_at(bytes_len);
		*buf = rest;

		let mut data = [0_u8; 32];
		data.copy_from_slice(front);

		let value: U256 = unsafe { core::mem::transmute(data) };

		Ok(value)
	}

	fn vec_from_bytes(len: u32, buf: &mut &[u8]) -> borsh::maybestd::io::Result<Option<borsh::maybestd::vec::Vec<Self>>> {
		let bytes_len = core::mem::size_of::<U256>() * (len as usize);

		let mut vec = borsh::maybestd::vec::Vec::with_capacity(len as usize);

		let (front, rest) = buf.split_at(bytes_len);
		*buf = rest;

		unsafe {
			let ptr = vec.as_mut_ptr() as *mut u8;
			ptr.copy_from_nonoverlapping(front.as_ptr(), bytes_len);

			vec.set_len(len as usize);
		}

		Ok(Some(vec))
	}

	fn copy_from_bytes(buf: &mut &[u8], out: &mut [Self]) -> borsh::maybestd::io::Result<bool> {
		let bytes_len = core::mem::size_of::<U256>() * out.len();

		let (front, rest) = buf.split_at(bytes_len);
		*buf = rest;

		let (_, out, _) = unsafe { out.align_to_mut::<u8>() };
		out.copy_from_slice(front);

		Ok(true)
	}
}


impl U256 {
	pub fn into_big_endian_fast(self, buffer: &mut [u8]) {
		let (low, high): (u128, u128) = unsafe { core::mem::transmute(self) };

		buffer[..16].copy_from_slice(&high.to_be_bytes());
		buffer[16..].copy_from_slice(&low.to_be_bytes());
	}

	#[must_use]
	pub fn from_big_endian_fast(buffer: &[u8]) -> U256 {
		assert!(32 >= buffer.len());

		let mut data = [0_u8; 32];
		data[32 - buffer.len()..32].copy_from_slice(buffer);
		
		unsafe {
			let (high, low): (u128, u128) =  core::mem::transmute(data);
			core::mem::transmute((u128::from_be(low), u128::from_be(high)))
		}
	}
}

impl From<U256> for H256 {
	fn from(value: U256) -> H256 {
		let mut h = H256::default();
		value.into_big_endian_fast(&mut h[..]);
		h
	}
}

impl From<U256> for H160 {
	fn from(value: U256) -> H160 {
		H256::from(value).into()
	}
}
