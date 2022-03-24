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


#[macro_export]
macro_rules! impl_uint_serde {
	($name: ident, $len: expr) => {
		impl $crate::serde::Serialize for $name {
			fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
			where
				S: $crate::serde::Serializer,
			{
				
			}
		}

		impl<'de> $crate::serde::Deserialize<'de> for $name {
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
			where
				D: $crate::serde::Deserializer<'de>,
			{
				
			}
		}
	};
}

/// Add Serde serialization support to a fixed-sized hash type created by `construct_fixed_hash!`.
#[macro_export]
macro_rules! impl_fixed_hash_serde {
	($name: ident) => {
		impl serde::Serialize for $name {
			fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
			where
				S: serde::Serializer,
			{
				serializer.serialize_bytes(self.as_bytes())
			}
		}

		impl<'de> serde::Deserialize<'de> for $name {
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
			where
				D: serde::Deserializer<'de>,
			{
				struct Visitor;
				impl<'de> serde::de::Visitor<'de> for Visitor {
					type Value = $name;

					fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
						formatter.write_str(stringify!($name))
					}

					fn visit_bytes<E: serde::de::Error>(self, v: &[u8]) -> Result<Self::Value, E>
					{
						let mut data = $name::default();
						data.as_bytes_mut().copy_from_slice(v);

						Ok(data)
					}
				}

				deserializer.deserialize_bytes(Visitor)
			}
		}
	};
}

#[cfg(feature = "with-serde")]
impl_fixed_hash_serde!(H160);

#[cfg(feature = "with-serde")]
impl_fixed_hash_serde!(H256);


impl serde::Serialize for U256 {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let data: [u8; 32] = unsafe { core::mem::transmute_copy(self) };
		serializer.serialize_bytes(&data)
	}
}

impl<'de> serde::Deserialize<'de> for U256 {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct Visitor;
		impl<'de> serde::de::Visitor<'de> for Visitor {
			type Value = U256;

			fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
				formatter.write_str("U256")
			}

			fn visit_bytes<E: serde::de::Error>(self, v: &[u8]) -> Result<Self::Value, E>
			{
				let mut data = [0_u8; 32];
				data.copy_from_slice(v);

				let value: U256 = unsafe { core::mem::transmute(data) };
				Ok(value)
			}
		}

		deserializer.deserialize_bytes(Visitor)
	}
}


impl U256 {
	pub fn into_big_endian_fast(self, buffer: &mut [u8]) {
		let data: [u8; 32] = unsafe { core::mem::transmute(self) };
		
		let buffer = &mut buffer[0..32];
		buffer.copy_from_slice(&data[..]);
		buffer.reverse();
	}

	#[must_use]
	pub fn from_big_endian_fast(buffer: &[u8]) -> U256 {
		assert!(32 >= buffer.len());

		let mut data = [0_u8; 32];

		data[32 - buffer.len()..32].copy_from_slice(buffer);
		data.reverse();

		unsafe { core::mem::transmute(data) }
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
