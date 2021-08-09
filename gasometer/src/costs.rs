use crate::consts::*;
use evm_core::{ExitError, H256, U256};
use evm_runtime::CONFIG;

pub fn call_extra_check(gas: U256, after_gas: u64) -> Result<(), ExitError> {
	if CONFIG.err_on_call_with_more_gas && U256::from(after_gas) < gas {
		Err(ExitError::OutOfGas)
	} else {
		Ok(())
	}
}

pub fn suicide_refund(already_removed: bool) -> i64 {
	if already_removed {
		0
	} else {
		R_SUICIDE
	}
}

pub fn sstore_refund(original: H256, current: H256, new: H256) -> i64 {
	if CONFIG.sstore_gas_metering {
		if current == new {
			0
		} else {
			if original == current && new == H256::default() {
				CONFIG.refund_sstore_clears
			} else {
				let mut refund = 0;
				if original != H256::default() {
					if current == H256::default() {
						refund -= CONFIG.refund_sstore_clears;
					} else if new == H256::default() {
						refund += CONFIG.refund_sstore_clears;
					}
				}

				if original == new {
					if original == H256::default() {
						refund += (CONFIG.gas_sstore_set - CONFIG.gas_sload) as i64;
					} else {
						refund += (CONFIG.gas_sstore_reset - CONFIG.gas_sload) as i64;
					}
				}

				refund
			}
		}
	} else {
		if current != H256::default() && new == H256::default() {
			CONFIG.refund_sstore_clears
		} else {
			0
		}
	}
}

pub fn create2_cost(len: U256) -> Result<u64, ExitError> {
	let base = U256::from(G_CREATE);
	// ceil(len / 32.0)
	let sha_addup_base = len / U256::from(32) +
		if len % U256::from(32) == U256::zero() { U256::zero() } else { U256::one() };
	let sha_addup = U256::from(G_SHA3WORD).checked_mul(sha_addup_base)
		.ok_or(ExitError::OutOfGas)?;
	let gas = base.checked_add(sha_addup).ok_or(ExitError::OutOfGas)?;

	if gas > U256::from(u64::MAX) {
		return Err(ExitError::OutOfGas)
	}

	Ok(gas.as_u64())
}

pub fn exp_cost(power: U256) -> Result<u64, ExitError> {
	if power == U256::zero() {
		Ok(G_EXP)
	} else {
		let gas = U256::from(G_EXP)
			.checked_add(
				U256::from(CONFIG.gas_expbyte)
					.checked_mul(U256::from(crate::utils::log2floor(power) / 8 + 1))
					.ok_or(ExitError::OutOfGas)?
			)
			.ok_or(ExitError::OutOfGas)?;

		if gas > U256::from(u64::MAX) {
			return Err(ExitError::OutOfGas)
		}

		Ok(gas.as_u64())
	}
}

pub fn verylowcopy_cost(len: U256) -> Result<u64, ExitError> {
	let wordd = len / U256::from(32);
	let wordr = len % U256::from(32);

	let gas = U256::from(G_VERYLOW).checked_add(
		U256::from(G_COPY).checked_mul(
			if wordr == U256::zero() {
				wordd
			} else {
				wordd + U256::one()
			}
		).ok_or(ExitError::OutOfGas)?
	).ok_or(ExitError::OutOfGas)?;

	if gas > U256::from(u64::MAX) {
		return Err(ExitError::OutOfGas)
	}

	Ok(gas.as_u64())
}

pub fn extcodecopy_cost(len: U256) -> Result<u64, ExitError> {
	let wordd = len / U256::from(32);
	let wordr = len % U256::from(32);

	let gas = U256::from(CONFIG.gas_ext_code).checked_add(
		U256::from(G_COPY).checked_mul(
			if wordr == U256::zero() {
				wordd
			} else {
				wordd + U256::one()
			}
		).ok_or(ExitError::OutOfGas)?
	).ok_or(ExitError::OutOfGas)?;

	if gas > U256::from(u64::MAX) {
		return Err(ExitError::OutOfGas)
	}

	Ok(gas.as_u64())
}

pub fn log_cost(n: u8, len: U256) -> Result<u64, ExitError> {
	let gas = U256::from(G_LOG)
		.checked_add(U256::from(G_LOGDATA).checked_mul(len).ok_or(ExitError::OutOfGas)?)
		.ok_or(ExitError::OutOfGas)?
		.checked_add(U256::from(G_LOGTOPIC * n as u64))
		.ok_or(ExitError::OutOfGas)?;

	if gas > U256::from(u64::MAX) {
		return Err(ExitError::OutOfGas)
	}

	Ok(gas.as_u64())
}

pub fn sha3_cost(len: U256) -> Result<u64, ExitError> {
	let wordd = len / U256::from(32);
	let wordr = len % U256::from(32);

	let gas = U256::from(G_SHA3).checked_add(
		U256::from(G_SHA3WORD).checked_mul(
			if wordr == U256::zero() {
				wordd
			} else {
				wordd + U256::one()
			}
		).ok_or(ExitError::OutOfGas)?
	).ok_or(ExitError::OutOfGas)?;

	if gas > U256::from(u64::MAX) {
		return Err(ExitError::OutOfGas)
	}

	Ok(gas.as_u64())
}

pub fn sstore_cost(original: H256, current: H256, new: H256, gas: u64) -> Result<u64, ExitError> {
	if CONFIG.sstore_gas_metering {
		if CONFIG.sstore_revert_under_stipend {
			if gas < CONFIG.call_stipend {
				return Err(ExitError::OutOfGas)
			}
		}

		Ok(if new == current {
			CONFIG.gas_sload
		} else {
			if original == current {
				if original == H256::zero() {
					CONFIG.gas_sstore_set
				} else {
					CONFIG.gas_sstore_reset
				}
			} else {
				CONFIG.gas_sload
			}
		})
	} else {
		Ok(if current == H256::zero() && new != H256::zero() {
			CONFIG.gas_sstore_set
		} else {
			CONFIG.gas_sstore_reset
		})
	}
}

pub fn suicide_cost(value: U256, target_exists: bool) -> u64 {
	let eip161 = !CONFIG.empty_considered_exists;
	let should_charge_topup = if eip161 {
		value != U256::zero() && !target_exists
	} else {
		!target_exists
	};

	let suicide_gas_topup = if should_charge_topup {
		CONFIG.gas_suicide_new_account
	} else {
		0
	};

	CONFIG.gas_suicide + suicide_gas_topup
}

pub fn call_cost(
	value: U256,
	is_call_or_callcode: bool,
	is_call_or_staticcall: bool,
	new_account: bool,
) -> u64 {
	let transfers_value = value != U256::default();
	CONFIG.gas_call +
		xfer_cost(is_call_or_callcode, transfers_value) +
		new_cost(is_call_or_staticcall, new_account, transfers_value)
}

fn xfer_cost(
	is_call_or_callcode: bool,
	transfers_value: bool
) -> u64 {
	if is_call_or_callcode && transfers_value {
		G_CALLVALUE
	} else {
		0
	}
}

fn new_cost(
	is_call_or_staticcall: bool,
	new_account: bool,
	transfers_value: bool,
) -> u64 {
	let eip161 = !CONFIG.empty_considered_exists;
	if is_call_or_staticcall {
		if eip161 {
			if transfers_value && new_account {
				G_NEWACCOUNT
			} else {
				0
			}
		} else if new_account {
			G_NEWACCOUNT
		} else {
			0
		}
	} else {
		0
	}
}
