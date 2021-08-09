#![allow(clippy::let_underscore_drop)]

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;
use core::convert::Infallible;
use evm_runtime::CONFIG;

use crate::{
	Capture, Context, CreateScheme, ExitError, ExitReason, ExitSucceed, H160,
	H256, Handler, Opcode, Runtime, Stack, Transfer, Valids, U256,
};
use crate::backend::{Apply, Backend, Basic, Log};
use crate::gasometer::{self, Gasometer};


/// Account definition for the stack-based executor.
#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct StackAccount {
	/// Basic account information, including nonce and balance.
	pub basic: Basic,
	/// Code. `None` means the code is currently unknown.
	pub code: Option<Vec<u8>>,
	/// Valids. `None` means the code is currently unknown.
	pub valids: Option<Vec<u8>>,
	/// Storage. Not inserted values mean it is currently known, but not empty.
	pub storage: BTreeMap<U256, U256>,
	/// Whether the storage in the database should be reset before storage
	/// values are applied.
	pub reset_storage: bool,
}

type PrecompileOutput = (ExitSucceed, Vec<u8>, u64);
type PrecompileResult = Option<Result<PrecompileOutput, ExitError>>;
type PrecompileFn = fn(H160, &[u8], Option<u64>) -> PrecompileResult;

/// Stack-based executor.
#[derive(Clone)]
pub struct StackExecutor<'backend, B> {
	backend: &'backend B,
	gasometer: Gasometer,
	state: BTreeMap<H160, StackAccount>,
	deleted: BTreeSet<H160>,
	logs: Vec<Log>,
	precompile: PrecompileFn,
	is_static: bool,
	depth: Option<usize>,
}

const fn no_precompile(
	_address: H160,
	_input: &[u8],
	_target_gas: Option<u64>
) -> PrecompileResult {
	None
}

impl<'backend, B: 'backend + Backend> StackExecutor<'backend, B> {
	/// Create a new stack-based executor.
	pub fn new(
		backend: &'backend B,
		gas_limit: u64,
	) -> Self {
		Self::new_with_precompile(backend, gas_limit, no_precompile)
	}

	/// Create a new stack-based executor with given precompiles.
	pub fn new_with_precompile(
		backend: &'backend B,
		gas_limit: u64,
		precompile: PrecompileFn,
	) -> Self {
		Self {
			backend,
			gasometer: Gasometer::new(gas_limit),
			state: BTreeMap::new(),
			deleted: BTreeSet::new(),
			logs: Vec::new(),
			precompile,
			is_static: false,
			depth: None,
		}
	}

	/// Create a substate executor from the current executor.
	#[must_use]
	pub fn substate(&self, gas_limit: u64, is_static: bool) -> StackExecutor<'backend, B> {
		Self {
			backend: self.backend,
			gasometer: Gasometer::new(gas_limit),
			state: self.state.clone(),
			deleted: self.deleted.clone(),
			logs: self.logs.clone(),
			precompile: self.precompile,
			is_static: is_static || self.is_static,
			depth: match self.depth {
				None => Some(0),
				Some(n) => Some(n + 1),
			},
		}
	}

	/// Execute the runtime until it returns.
	pub fn execute(&mut self, runtime: &mut Runtime) -> ExitReason {
		match runtime.run(u64::max_value(), self).1 {
			Capture::Exit(s) => s,
			Capture::Trap(_) => unreachable!("Trap is Infallible"),
		}
	}

	/// Get remaining gas.
	#[must_use]
	pub fn gas(&self) -> u64 {
		self.gasometer.gas() // 12341234
	}

	/// Merge a substate executor that succeeded.
	pub fn merge_succeed<OB>(
		&mut self,
		mut substate: StackExecutor<OB>
	) -> Result<(), ExitError> {
		self.logs.append(&mut substate.logs);
		self.deleted.append(&mut substate.deleted);
		self.state = substate.state;

		self.gasometer.record_stipend(substate.gasometer.gas())?;
		self.gasometer.record_refund(substate.gasometer.refunded_gas())?;
		Ok(())
	}

	/// Merge a substate executor that reverted.
	pub fn merge_revert<OB>(
		&mut self,
		mut substate: StackExecutor<OB>
	) -> Result<(), ExitError> {
		self.logs.append(&mut substate.logs);

		self.gasometer.record_stipend(substate.gasometer.gas())?;
		Ok(())
	}

	/// Merge a substate executor that failed.
	pub fn merge_fail<OB>(
		&mut self,
		mut substate: StackExecutor<OB>
	) -> Result<(), ExitError> {
		self.logs.append(&mut substate.logs);

		Ok(())
	}

	/// Execute a `CREATE` transaction.
	pub fn transact_create(
		&mut self,
		caller: H160,
		value: U256,
		init_code: Vec<u8>,
		gas_limit: u64,
	) -> ExitReason {
		let transaction_cost = gasometer::create_transaction_cost(&init_code);
		match self.gasometer.record_transaction(transaction_cost) {
			Ok(()) => (),
			Err(e) => return e.into(),
		}

		match self.create_inner(
			caller,
			CreateScheme::Legacy { caller },
			value,
			init_code,
			Some(gas_limit),
			false,
		) {
			Capture::Exit((s, _, _)) => s,
			Capture::Trap(_) => unreachable!(),
		}
	}

	/// Execute a `CREATE2` transaction.
	pub fn transact_create2(
		&mut self,
		caller: H160,
		value: U256,
		init_code: Vec<u8>,
		salt: H256,
		gas_limit: u64,
	) -> ExitReason {
		let transaction_cost = gasometer::create_transaction_cost(&init_code);
		match self.gasometer.record_transaction(transaction_cost) {
			Ok(()) => (),
			Err(e) => return e.into(),
		}
		let code_hash = self.backend.keccak256_h256(&init_code); //H256::from_slice(Keccak256::digest(&init_code).as_slice());

		match self.create_inner(
			caller,
			CreateScheme::Create2 { caller, code_hash, salt },
			value,
			init_code,
			Some(gas_limit),
			false,
		) {
			Capture::Exit((s, _, _)) => s,
			Capture::Trap(_) => unreachable!(),
		}
	}

	/// Execute a `CALL` transaction.
	pub fn transact_call(
		&mut self,
		caller: H160,
		address: H160,
		value: U256,
		data: Vec<u8>,
		gas_limit: u64,
	) -> (ExitReason, Vec<u8>) {
		let transaction_cost = gasometer::call_transaction_cost(&data);
		match self.gasometer.record_transaction(transaction_cost) {
			Ok(()) => (),
			Err(e) => return (e.into(), Vec::new()),
		}

		self.account_mut(caller).basic.nonce += U256::one();

		let context = Context {
			caller,
			address,
			apparent_value: value,
		};

		match self.call_inner(address, Some(Transfer {
			source: caller,
			target: address,
			value
		}), data, Some(gas_limit), false, false, false, context) {
			Capture::Exit((s, v)) => (s, v),
			Capture::Trap(_) => unreachable!(),
		}
	}

	/// Get used gas for the current executor.
	#[must_use]
	#[allow(clippy::cast_sign_loss)]
	pub fn used_gas(&self) -> u64 {
		let rg = self.gasometer.refunded_gas();
		assert!(rg >= 0);
		let tug = self.gasometer.total_used_gas();
		tug - core::cmp::min(tug / 2, rg as u64)
        // 0
	}

	/// Get fee needed for the current executor, given the price.
	#[must_use]
	pub fn fee(&self, price: U256) -> U256 {
		let used_gas = self.used_gas();
		U256::from(used_gas) * price
	}

	/// Deconstruct the executor, return state to be applied.
	#[must_use]
	pub fn deconstruct(
		self
	) -> (Vec::<Apply<BTreeMap<U256, U256>>>, Vec<Log>)
	{
		let mut applies = Vec::<Apply<BTreeMap<U256, U256>>>::new();

		for (address, account) in self.state {
			if self.deleted.contains(&address) {
				continue
			}

			applies.push(Apply::Modify {
				address,
				basic: account.basic,
				code_and_valids: account.code.zip(account.valids),
				storage: account.storage,
				reset_storage: account.reset_storage,
			});
		}

		for address in self.deleted {
			applies.push(Apply::Delete { address });
		}

		let logs = self.logs;

		(applies, logs)
	}

	/// Get mutable account reference.
	pub fn account_mut(&mut self, address: H160) -> &mut StackAccount {
		self.state.entry(address).or_insert(StackAccount {
			basic: self.backend.basic(address),
			code: None,
			valids: None,
			storage: BTreeMap::new(),
			reset_storage: false,
		})
	}

	/// Get account nonce.
	#[must_use]
	pub fn nonce(&self, address: H160) -> U256 {
		self.state.get(&address).map_or(self.backend.basic(address).nonce, |v| v.basic.nonce)
	}

	/// Withdraw balance from address.
	pub fn withdraw(&mut self, address: H160, balance: U256) -> Result<(), ExitError> {
		let source = self.account_mut(address);
		if source.basic.balance < balance {
			return Err(ExitError::OutOfFund)
		}
		source.basic.balance -= balance;

		Ok(())
	}

	/// Deposit balance to address.
	pub fn deposit(&mut self, address: H160, balance: U256) {
		let target = self.account_mut(address);
		target.basic.balance += balance;
	}

	/// Transfer balance with the given struct.
	pub fn transfer(&mut self, transfer: &Transfer) -> Result<(), ExitError> {
		self.withdraw(transfer.source, transfer.value)?;
		self.deposit(transfer.target, transfer.value);

		Ok(())
	}

	/// Get the create address from given scheme.
	#[must_use]
	pub fn create_address(&self, scheme: CreateScheme) -> H160 {
		match scheme {
			CreateScheme::Create2 { caller, code_hash, salt } => {
				self.backend.keccak256_h256_v(&[&[0xff], &caller[..], &salt[..], &code_hash[..]]).into()
			},
			CreateScheme::Legacy { caller } => {
				let nonce = self.nonce(caller);
				let mut stream = rlp::RlpStream::new_list(2);
				stream.append(&caller);
				stream.append(&nonce);
				//H256::from_slice(Keccak256::digest(&stream.out()).as_slice()).into()
				self.backend.keccak256_h256(&stream.out()).into()
			},
			CreateScheme::Fixed(naddress) => {
				naddress
			},
		}
	}

	
	#[allow(clippy::too_many_lines)]
	fn create_inner(
		&mut self,
		caller: H160,
		scheme: CreateScheme,
		value: U256,
		init_code: Vec<u8>,
		target_gas: Option<u64>,
		take_l64: bool,
	) -> Capture<(ExitReason, Option<H160>, Vec<u8>), Infallible> {
		macro_rules! try_or_fail {
			( $e:expr ) => {
				match $e {
					Ok(v) => v,
					Err(e) => return Capture::Exit((e.into(), None, Vec::new())),
				}
			}
		}

		const fn l64(gas: u64) -> u64 {
			gas - gas / 64
		}

		if let Some(depth) = self.depth {
			if depth + 1 > CONFIG.call_stack_limit {
				return Capture::Exit((ExitError::CallTooDeep.into(), None, Vec::new()))
			}
		}

		if self.balance(caller) < value {
			return Capture::Exit((ExitError::OutOfFund.into(), None, Vec::new()))
		}

		let mut after_gas = self.gasometer.gas(); // 0;
		if take_l64 && CONFIG.call_l64_after_gas {
			after_gas = l64(after_gas);
		}
		let target_gas = target_gas.unwrap_or(after_gas);

		let gas_limit = core::cmp::min(after_gas, target_gas);
		try_or_fail!(self.gasometer.record_cost(gas_limit));

		let address = self.create_address(scheme);
                self.backend.create(&scheme, &address);
		self.account_mut(caller).basic.nonce += U256::one();

		let mut substate = self.substate(gas_limit, false);
		{
			if let Some(code) = substate.account_mut(address).code.as_ref() {
				if !code.is_empty() {
					let _ = self.merge_fail(substate);
					return Capture::Exit((ExitError::CreateCollision.into(), None, Vec::new()))
				}
			} else  {
				let code = substate.backend.code(address);
				substate.account_mut(address).code = Some(code.clone());

				if !code.is_empty() {
					let _ = self.merge_fail(substate);
					return Capture::Exit((ExitError::CreateCollision.into(), None, Vec::new()))
				}
			}

			if substate.account_mut(address).basic.nonce > U256::zero() {
				let _ = self.merge_fail(substate);
				return Capture::Exit((ExitError::CreateCollision.into(), None, Vec::new()))
			}

			substate.account_mut(address).reset_storage = true;
			substate.account_mut(address).storage = BTreeMap::new();
		}

		let context = Context {
			address,
			caller,
			apparent_value: value,
		};
		let transfer = Transfer {
			source: caller,
			target: address,
			value,
		};
		match substate.transfer(&transfer) {
			Ok(()) => (),
			Err(e) => {
				let _ = self.merge_revert(substate);
				return Capture::Exit((ExitReason::Error(e), None, Vec::new()))
			},
		}

		if CONFIG.create_increase_nonce {
			substate.account_mut(address).basic.nonce += U256::one();
		}

		let valids = Valids::compute(&init_code);
		let mut runtime = Runtime::new(
			init_code,
			valids,
			Vec::new(),
			context,
		);

		let reason = substate.execute(&mut runtime);
		//log::debug!(target: "evm", "Create execution using address {}: {:?}", address, reason);

		match reason {
			ExitReason::Succeed(s) => {
				let out = runtime.machine().return_value();

				if let Some(limit) = CONFIG.create_contract_limit {
					if out.len() > limit {
						substate.gasometer.fail();
						let _ = self.merge_fail(substate);
						return Capture::Exit((ExitError::CreateContractLimit.into(), None, Vec::new()))
					}
				}

				match substate.gasometer.record_deposit(out.len()) {
					Ok(()) => {
						let e = self.merge_succeed(substate);
						let entry: &mut _ = self.state.entry(address).or_insert_with(Default::default);
						entry.valids = Some(Valids::compute(&out));
						entry.code = Some(out);
						try_or_fail!(e);
						Capture::Exit((ExitReason::Succeed(s), Some(address), Vec::new()))
					},
					Err(e) => {
						let _ = self.merge_fail(substate);
						Capture::Exit((ExitReason::Error(e), None, Vec::new()))
					},
				}
			},
			ExitReason::Error(e) => {
				substate.gasometer.fail();
				let _ = self.merge_fail(substate);
				Capture::Exit((ExitReason::Error(e), None, Vec::new()))
			},
			ExitReason::Revert(e) => {
				let _ = self.merge_revert(substate);
				Capture::Exit((ExitReason::Revert(e), None, runtime.machine().return_value()))
			},
			ExitReason::Fatal(e) => {
				self.gasometer.fail();
				Capture::Exit((ExitReason::Fatal(e), None, Vec::new()))
			},
			ExitReason::StepLimitReached => { unreachable!() }
		}
	}

	#[allow(clippy::too_many_arguments)]
	#[allow(clippy::too_many_lines)]
	fn call_inner(
		&mut self,
		code_address: H160,
		transfer: Option<Transfer>,
		input: Vec<u8>,
		target_gas: Option<u64>,
		is_static: bool,
		take_l64: bool,
		take_stipend: bool,
		context: Context,
	) -> Capture<(ExitReason, Vec<u8>), Infallible> {
		macro_rules! try_or_fail {
			( $e:expr ) => {
				match $e {
					Ok(v) => v,
					Err(e) => return Capture::Exit((e.into(), Vec::new())),
				}
			}
		}

		const fn l64(gas: u64) -> u64 {
			gas - gas / 64
		}

		let mut after_gas = self.gasometer.gas(); // 0;
		if take_l64 && CONFIG.call_l64_after_gas {
			after_gas = l64(after_gas);
		}

		let target_gas = target_gas.unwrap_or(after_gas);
		let mut gas_limit = core::cmp::min(target_gas, after_gas);

		try_or_fail!(self.gasometer.record_cost(gas_limit));

		if let Some(transfer) = transfer.as_ref() {
			if take_stipend && transfer.value != U256::zero() {
				gas_limit = gas_limit.saturating_add(CONFIG.call_stipend);
			}
		}

		let code = self.code(code_address);
		let valids = self.valids(code_address);

		let mut substate = self.substate(gas_limit, is_static);
		substate.account_mut(context.address);

		if let Some(depth) = self.depth {
			if depth + 1 > CONFIG.call_stack_limit {
				let _ = self.merge_revert(substate);
				return Capture::Exit((ExitError::CallTooDeep.into(), Vec::new()))
			}
		}

		if let Some(transfer) = transfer {
			match substate.transfer(&transfer) {
				Ok(()) => (),
				Err(e) => {
					let _ = self.merge_revert(substate);
					return Capture::Exit((ExitReason::Error(e), Vec::new()))
				},
			}
		}

		if let Some(ret) = (substate.precompile)(code_address, &input, Some(gas_limit)) {
			return match ret {
				Ok((s, out, cost)) => {
					let _ = substate.gasometer.record_cost(cost);
					let _ = self.merge_succeed(substate);
					Capture::Exit((ExitReason::Succeed(s), out))
				},
				Err(e) => {
					let _ = self.merge_fail(substate);
					Capture::Exit((ExitReason::Error(e), Vec::new()))
				},
			}
		}

		let hook_res = self.backend.call_inner(code_address, transfer, input.clone(), Some(target_gas), is_static, take_l64, take_stipend);
		if let Some(hook_res) = hook_res {
			match &hook_res {
				Capture::Exit((reason, _return_data)) => {
					match reason {
						ExitReason::Succeed(_) => {
							let _ = self.merge_succeed(substate);
						},
						ExitReason::Revert(_) => {
							let _ = self.merge_revert(substate);
						},
						ExitReason::Error(_) => {
							let _ = self.merge_fail(substate);
						},
						ExitReason::Fatal(_) => {
						},
						ExitReason::StepLimitReached => { unreachable!() }
					}
				},
				Capture::Trap(_interrupt) => {
				},
			}
			return hook_res;
		}

		let mut runtime = Runtime::new(
			code,
			valids,
			input,
			context,
		);

		let reason = substate.execute(&mut runtime);
		//log::debug!(target: "evm", "Call execution using address {}: {:?}", code_address, reason);

		match reason {
			ExitReason::Succeed(s) => {
				let _ = self.merge_succeed(substate);
				Capture::Exit((ExitReason::Succeed(s), runtime.machine().return_value()))
			},
			ExitReason::Error(e) => {
				let _ = self.merge_fail(substate);
				Capture::Exit((ExitReason::Error(e), Vec::new()))
			},
			ExitReason::Revert(e) => {
				let _ = self.merge_revert(substate);
				Capture::Exit((ExitReason::Revert(e), runtime.machine().return_value()))
			},
			ExitReason::Fatal(e) => {
				self.gasometer.fail();
				Capture::Exit((ExitReason::Fatal(e), Vec::new()))
			},
			ExitReason::StepLimitReached => { unreachable!() }
		}
	}
}

impl<'backend, B: Backend> Handler for StackExecutor<'backend, B> {
	type CreateInterrupt = Infallible;
	type CreateFeedback = Infallible;
	type CallInterrupt = Infallible;
	type CallFeedback = Infallible;

	fn keccak256_h256(&self, data: &[u8]) -> H256 {
		self.backend.keccak256_h256(data)
	}

	fn balance(&self, address: H160) -> U256 {
		self.state.get(&address).map_or(self.backend.basic(address).balance, |v| v.basic.balance)
	}

	fn code_size(&self, address: H160) -> U256 {
		U256::from(
			self.state.get(&address).and_then(|v| v.code.as_ref().map(Vec::len))
				.unwrap_or_else(|| self.backend.code_size(address))
		)
	}

	fn code_hash(&self, address: H160) -> H256 {
		if !self.exists(address) {
			return H256::default()
		}

		let (balance, nonce, code_size) = self.state.get(&address).map_or_else(|| {
			let basic = self.backend.basic(address);
			(basic.balance, basic.nonce, U256::from(self.backend.code_size(address)))
		}, |account| 
			(
				account.basic.balance, account.basic.nonce,
				account.code.as_ref().map_or(self.code_size(address), |c| U256::from(c.len()))
			)
		);

		if balance == U256::zero() && nonce == U256::zero() && code_size == U256::zero() {
			return H256::default()
		}

		let value = self.state.get(&address).and_then(|v| {
			v.code.as_ref().map(|c| {
				//H256::from_slice(Keccak256::digest(&c).as_slice())
				self.backend.keccak256_h256(c)
			})
		}).unwrap_or_else(|| self.backend.code_hash(address));
		value
	}

	fn code(&self, address: H160) -> Vec<u8> {
		self.state.get(&address).and_then(|v| {
			v.code.clone()
		}).unwrap_or_else(|| self.backend.code(address))
	}

	fn valids(&self, address: H160) -> Vec<u8> {
		self.state.get(&address).and_then(|v| {
			v.valids.clone()
		}).unwrap_or_else(|| self.backend.valids(address))
	}

	fn storage(&self, address: H160, index: U256) -> U256 {
		self.state.get(&address)
			.and_then(|v| {
				let s = v.storage.get(&index).cloned();

				if v.reset_storage {
					Some(s.unwrap_or_else(U256::zero))
				} else {
					s
				}

			})
			.unwrap_or_else(|| self.backend.storage(address, index))
	}

	fn original_storage(&self, address: H160, index: U256) -> U256 {
		if let Some(account) = self.state.get(&address) {
			if account.reset_storage {
				return U256::zero()
			}
		}
		self.backend.storage(address, index)
	}

	#[allow(clippy::option_if_let_else)]
	#[allow(clippy::map_unwrap_or)]
	fn exists(&self, address: H160) -> bool {
		if CONFIG.empty_considered_exists {
			self.state.get(&address).is_some() || self.backend.exists(address)
		} else if let Some(account) = self.state.get(&address) {
			account.basic.nonce != U256::zero() ||
				account.basic.balance != U256::zero() ||
				account.code.as_ref().map(|c| !c.is_empty()).unwrap_or(false) ||
				!self.backend.code(address).is_empty()
		} else {
			self.state.get(&address).map_or_else(||
					self.backend.basic(address).nonce != U256::zero() ||
					self.backend.basic(address).balance != U256::zero() ||
					!self.backend.code(address).is_empty(), 
				|account| 
					account.basic.nonce != U256::zero() ||
					account.basic.balance != U256::zero() ||
					account.code.as_ref().map_or(false, |c| !c.is_empty()) ||
					!self.backend.code(address).is_empty()
			)
		}
	}

	fn gas_left(&self) -> U256 { U256::from(self.gasometer.gas()) } // { U256::one() }

	fn gas_price(&self) -> U256 { self.backend.gas_price() }
	fn origin(&self) -> H160 { self.backend.origin() }
	fn block_hash(&self, number: U256) -> H256 { self.backend.block_hash(number) }
	fn block_number(&self) -> U256 { self.backend.block_number() }
	fn block_coinbase(&self) -> H160 { self.backend.block_coinbase() }
	fn block_timestamp(&self) -> U256 { self.backend.block_timestamp() }
	fn block_difficulty(&self) -> U256 { self.backend.block_difficulty() }
	fn block_gas_limit(&self) -> U256 { self.backend.block_gas_limit() }
	fn chain_id(&self) -> U256 { self.backend.chain_id() }

	fn deleted(&self, address: H160) -> bool { self.deleted.contains(&address) }

	fn set_storage(&mut self, address: H160, index: U256, value: U256) -> Result<(), ExitError> {
		self.account_mut(address).storage.insert(index, value);

		Ok(())
	}

	fn log(&mut self, address: H160, topics: Vec<H256>, data: Vec<u8>) -> Result<(), ExitError> {
		self.logs.push(Log {
			address, topics, data
		});

		Ok(())
	}

	fn mark_delete(&mut self, address: H160, target: H160) -> Result<(), ExitError> {
		let balance = self.balance(address);

		self.transfer(&Transfer {
			source: address,
			target,
			value: balance
		})?;
		self.account_mut(address).basic.balance = U256::zero();

		self.deleted.insert(address);

		Ok(())
	}

	fn create(
		&mut self,
		caller: H160,
		scheme: CreateScheme,
		value: U256,
		init_code: Vec<u8>,
		target_gas: Option<u64>,
	) -> Capture<(ExitReason, Option<H160>, Vec<u8>), Self::CreateInterrupt> {
		self.create_inner(caller, scheme, value, init_code, target_gas, true)
	}

	fn call(
		&mut self,
		code_address: H160,
		transfer: Option<Transfer>,
		input: Vec<u8>,
		target_gas: Option<u64>,
		is_static: bool,
		context: Context,
	) -> Capture<(ExitReason, Vec<u8>), Self::CallInterrupt> {
		self.call_inner(code_address, transfer, input, target_gas, is_static, true, true, context)
	}

	fn pre_validate(
		&mut self,
		context: &Context,
		opcode: Opcode,
		stack: &Stack,
	) -> Result<(), ExitError> {
		if let Some(cost) = gasometer::static_opcode_cost(opcode) {
			self.gasometer.record_cost(cost)?;
		} else {
			let is_static = self.is_static;
			let (gas_cost, memory_cost) = gasometer::dynamic_opcode_cost(
				context.address,
				opcode,
				stack,
				is_static,
				self,
			)?;
			self.gasometer.record_dynamic_cost(gas_cost, memory_cost)?;
		}

		Ok(())
	}
}
