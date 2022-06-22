//! # EVM backends
//!
//! Backends store state information of the VM, and exposes it to runtime.

extern crate alloc;

use alloc::vec::Vec;
use core::convert::Infallible;
use evm_core::{CreateScheme, Transfer};
use crate::{Capture, ExitReason, H160, H256, U256};

/// Basic account information.
#[derive(Clone, Eq, PartialEq, Debug, Default)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Basic {
	/// Account balance.
	pub balance: U256,
	/// Account nonce.
	pub nonce: U256,
}

/// Alias for `Vec<u8>`. This type alias is necessary for rlp-derive to work correctly.
pub type Bytes = alloc::vec::Vec<u8>;

/// Log info from contract
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Log {
	/// address
	pub address: H160,
	/// topics
	pub topics: Vec<H256>,
	/// data
	#[cfg_attr(feature = "with-serde", serde(with = "serde_bytes"))]
	pub data: Bytes,
}
//pub use ethereum::Log;

/// Apply state operation.
#[derive(Clone, Debug)]
pub enum Apply<I> {
	/// Modify or create at address.
	Modify {
		/// Address.
		address: H160,
		/// Nonce of the address.
		nonce: U256,
		/// Code. `None` means leaving it unchanged.
		code_and_valids: Option<(Vec<u8>, Vec<u8>)>,
		/// Storage iterator.
		storage: I,
		/// Whether storage should be wiped empty before applying the storage
		/// iterator.
		reset_storage: bool,
	},
	/// Delete address.
	Delete {
		/// Address.
		address: H160,
	},
}

/// EVM backend.
pub trait Backend {
	/// Gas price.
	fn gas_price(&self) -> U256;
	/// Origin.
	fn origin(&self) -> H160;
	/// Environmental block hash.
	fn block_hash(&self, number: U256) -> H256;
	/// Environmental block number.
	fn block_number(&self) -> U256;
	/// Environmental coinbase.
	fn block_coinbase(&self) -> H160;
	/// Environmental block timestamp.
	fn block_timestamp(&self) -> U256;
	/// Environmental block difficulty.
	fn block_difficulty(&self) -> U256;
	/// Environmental block gas limit.
	fn block_gas_limit(&self) -> U256;
	/// Environmental chain ID.
	fn chain_id(&self) -> U256;

	/// Whether account at address exists.
	fn exists(&self, address: H160) -> bool;
	/// Get basic account information.
	fn basic(&self, address: H160) -> Basic;
	/// Get account code hash.
	fn code_hash(&self, address: H160) -> H256;
	/// Get account code size.
	fn code_size(&self, address: H160) -> usize;
	/// Get account code.
	fn code(&self, address: H160) -> Vec<u8>;
	/// Get account code valids.
	fn valids(&self, address: H160) -> Vec<u8>;
	/// Get storage value of address at index.
	fn storage(&self, address: H160, index: U256) -> U256;

	/// Notification about create new address
	fn create(&self, scheme: &CreateScheme, address: &H160);

	/// Hook on Solidity's call
	#[allow(clippy::too_many_arguments)]
	fn call_inner(&self,
		code_address: H160,
		transfer: Option<Transfer>,
		input: Vec<u8>,
		target_gas: Option<u64>,
		is_static: bool,
		take_l64: bool,
		take_stipend: bool,
	) -> Option<Capture<(ExitReason, Vec<u8>), Infallible>>;

	/// Get keccak hash from slice
	fn keccak256_h256(&self, data: &[u8]) -> H256;

	/// Get keccak hash from array of slices
	fn keccak256_h256_v(&self, data: &[&[u8]]) -> H256;
}

/// EVM backend that can apply changes.
pub trait ApplyBackend {
	/// Apply given values and logs at backend.
	fn apply<A, I, L>(
		&mut self,
		values: A,
		logs: L,
		delete_empty: bool,
	) where
		A: IntoIterator<Item=Apply<I>>,
		I: IntoIterator<Item=(U256, U256)>,
		L: IntoIterator<Item=Log>;
}
