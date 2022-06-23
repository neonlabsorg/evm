//! Ethereum Virtual Machine implementation in Rust

#![deny(warnings)]
#![forbid(unsafe_code, missing_docs, unused_variables, unused_imports)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
	clippy::module_name_repetitions,
	clippy::missing_errors_doc,
	clippy::missing_panics_doc
)]

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use evm_core::*;
pub use evm_runtime::*;

pub mod backend;
