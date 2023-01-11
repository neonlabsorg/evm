//! Runtime layer for EVM.

#![deny(warnings)]
#![forbid(unsafe_code, unused_variables)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
	clippy::module_name_repetitions,
	clippy::missing_errors_doc,
	clippy::missing_panics_doc
)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "tracing"), forbid(unused_imports))]

extern crate alloc;


mod eval;
mod interrupt;
mod handler;

pub use evm_core::*;

pub use crate::interrupt::{Resolve, ResolveCall, ResolveCreate};
pub use crate::handler::Handler;
pub use crate::eval::{save_return_value, save_created_address, Control};

use alloc::vec::Vec;

/// EVM runtime.
///
/// The runtime wraps an EVM `Machine` with support of return data and context.
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(borsh::BorshSerialize, borsh::BorshDeserialize)]
pub struct Runtime {
	machine: Machine,
	status: Result<(), ExitReason>,
	#[cfg_attr(feature = "with-serde", serde(with = "serde_bytes"))]
	return_data_buffer: Vec<u8>,
	context: Context,
}

impl Runtime {
	/// Create a new runtime with given code and data.
	pub fn new(
		code: Vec<u8>,
		valids: Vec<u8>,
		data: Vec<u8>,
		context: Context,
	) -> Self {
		Self {
			machine: Machine::new(code, valids, data, CONFIG.stack_limit, CONFIG.memory_limit),
			status: Ok(()),
			return_data_buffer: Vec::new(),
			context,
		}
	}

	/// Get return data
	pub fn return_data(&self) -> &Vec<u8> {
		&self.return_data_buffer
	}

	/// Set return data
	pub fn set_return_data(&mut self, data: Vec<u8>) {
		self.return_data_buffer = data;
	}

	/// Get a reference to the machine.
	pub fn machine(&self) -> &Machine {
		&self.machine
	}

	/// Loop stepping the runtime until it stops.
	pub fn run<'a, H: Handler>(
		&'a mut self,
		max_steps: u64,
		handler: &mut H,
	) -> (u64, Capture<ExitReason, Resolve<'a, H>>) {
		if let Err(e) = self.status {
			return (0, Capture::Exit(e));
		}

		let mut steps = 0_u64;

		while steps < max_steps {
			let (steps_executed, capture) = {
				let context = &self.context;
				let pre_validate = |opcode, stack: &Stack| { handler.pre_validate(context, opcode, stack) };
				self.machine.run(max_steps - steps, pre_validate, &self.context)
			};
			steps += steps_executed;

			match capture {
				Capture::Exit(ExitReason::StepLimitReached) => {
					return (steps, Capture::Exit(ExitReason::StepLimitReached));
				},
				Capture::Exit(reason) => {
					self.status = Err(reason);
					return (steps, Capture::Exit(reason));
				},
				Capture::Trap(opcode) => {
					match eval::eval(self, opcode, handler) {
						eval::Control::Continue => {},
						eval::Control::CallInterrupt(interrupt) => {
							let resolve = ResolveCall::new(self);
							return (steps, Capture::Trap(Resolve::Call(interrupt, resolve)));
						},
						eval::Control::CreateInterrupt(interrupt) => {
							let resolve = ResolveCreate::new(self);
							return (steps, Capture::Trap(Resolve::Create(interrupt, resolve)));
						},
						eval::Control::Exit(exit) => {
							self.machine.exit(exit);
							self.status = Err(exit);
							return (steps, Capture::Exit(exit));
						},
					}
				},
			}
		}

		(steps, Capture::Exit(ExitReason::StepLimitReached))
	}
}

/// Runtime configuration.
#[derive(Clone, Debug)]
pub struct Config {
	/// Gas paid for extcode.
	pub gas_ext_code: u64,
	/// Gas paid for extcodehash.
	pub gas_ext_code_hash: u64,
	/// Gas paid for sstore set.
	pub gas_sstore_set: u64,
	/// Gas paid for sstore reset.
	pub gas_sstore_reset: u64,
	/// Gas paid for sstore refund.
	pub refund_sstore_clears: i64,
	/// Gas paid for BALANCE opcode.
	pub gas_balance: u64,
	/// Gas paid for SLOAD opcode.
	pub gas_sload: u64,
	/// Gas paid for SUICIDE opcode.
	pub gas_suicide: u64,
	/// Gas paid for SUICIDE opcode when it hits a new account.
	pub gas_suicide_new_account: u64,
	/// Gas paid for CALL opcode.
	pub gas_call: u64,
	/// Gas paid for EXP opcode for every byte.
	pub gas_expbyte: u64,
	/// Gas paid for a contract creation transaction.
	pub gas_transaction_create: u64,
	/// Gas paid for a message call transaction.
	pub gas_transaction_call: u64,
	/// Gas paid for zero data in a transaction.
	pub gas_transaction_zero_data: u64,
	/// Gas paid for non-zero data in a transaction.
	pub gas_transaction_non_zero_data: u64,
	/// EIP-1283.
	pub sstore_gas_metering: bool,
	/// EIP-1706.
	pub sstore_revert_under_stipend: bool,
	/// Whether to throw out of gas error when
	/// CALL/CALLCODE/DELEGATECALL requires more than maximum amount
	/// of gas.
	pub err_on_call_with_more_gas: bool,
	/// Take l64 for callcreate after gas.
	pub call_l64_after_gas: bool,
	/// Whether empty account is considered exists.
	pub empty_considered_exists: bool,
	/// Whether create transactions and create opcode increases nonce by one.
	pub create_increase_nonce: bool,
	/// Stack limit.
	pub stack_limit: usize,
	/// Memory limit.
	pub memory_limit: usize,
	/// Call limit.
	pub call_stack_limit: usize,
	/// Create contract limit.
	pub create_contract_limit: Option<usize>,
	/// Call stipend.
	pub call_stipend: u64,
	/// Has delegate call.
	pub has_delegate_call: bool,
	/// Has create2.
	pub has_create2: bool,
	/// Has revert.
	pub has_revert: bool,
	/// Has return data.
	pub has_return_data: bool,
	/// Has bitwise shifting.
	pub has_bitwise_shifting: bool,
	/// Has chain ID.
	pub has_chain_id: bool,
	/// Has self balance.
	pub has_self_balance: bool,
	/// Has ext code hash.
	pub has_ext_code_hash: bool,
	/// Whether the gasometer is running in estimate mode.
	pub estimate: bool,
}

pub const CONFIG: Config = Config::istanbul();

impl Config {
	/// Frontier hard fork configuration.
	pub const fn frontier() -> Config {
		Config {
			gas_ext_code: 20,
			gas_ext_code_hash: 20,
			gas_balance: 20,
			gas_sload: 50,
			gas_sstore_set: 20000,
			gas_sstore_reset: 5000,
			refund_sstore_clears: 15000,
			gas_suicide: 0,
			gas_suicide_new_account: 0,
			gas_call: 40,
			gas_expbyte: 10,
			gas_transaction_create: 21000,
			gas_transaction_call: 21000,
			gas_transaction_zero_data: 4,
			gas_transaction_non_zero_data: 68,
			sstore_gas_metering: false,
			sstore_revert_under_stipend: false,
			err_on_call_with_more_gas: true,
			empty_considered_exists: true,
			create_increase_nonce: false,
			call_l64_after_gas: false,
			stack_limit: 1024,
			memory_limit: usize::max_value(),
			call_stack_limit: 1024,
			create_contract_limit: None,
			call_stipend: 2300,
			has_delegate_call: false,
			has_create2: false,
			has_revert: false,
			has_return_data: false,
			has_bitwise_shifting: false,
			has_chain_id: false,
			has_self_balance: false,
			has_ext_code_hash: false,
			estimate: false,
		}
	}

	/// Istanbul hard fork configuration.
	pub const fn istanbul() -> Config {
		Config {
			gas_ext_code: 700,
			gas_ext_code_hash: 700,
			gas_balance: 700,
			gas_sload: 800,
			gas_sstore_set: 20000,
			gas_sstore_reset: 5000,
			refund_sstore_clears: 15000,
			gas_suicide: 5000,
			gas_suicide_new_account: 25000,
			gas_call: 700,
			gas_expbyte: 50,
			gas_transaction_create: 53000,
			gas_transaction_call: 21000,
			gas_transaction_zero_data: 4,
			gas_transaction_non_zero_data: 16,
			sstore_gas_metering: true,
			sstore_revert_under_stipend: true,
			err_on_call_with_more_gas: false,
			empty_considered_exists: false,
			create_increase_nonce: true,
			call_l64_after_gas: true,
			stack_limit: 1024,
			memory_limit: usize::max_value(),
			call_stack_limit: 1024,
			create_contract_limit: Some(0x6000),
			call_stipend: 2300,
			has_delegate_call: true,
			has_create2: true,
			has_revert: true,
			has_return_data: true,
			has_bitwise_shifting: true,
			has_chain_id: true,
			has_self_balance: true,
			has_ext_code_hash: true,
			estimate: false,
		}
	}

	/// Reference to default configuration
	pub fn default() -> &'static Config {
		&CONFIG
	}
}
