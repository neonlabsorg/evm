use core::cmp::min;
use super::Control;
use crate::{Machine, ExitError, ExitSucceed, ExitFatal, ExitRevert, H256, U256};

pub fn codesize(state: &mut Machine) -> Control {
	let size = U256::from(state.code.len());
	trace_op!("CodeSize: {}", size);
	push_u256!(state, size);
	Control::Continue(1)
}

pub fn codecopy(state: &mut Machine) -> Control {
	pop_u256!(state, memory_offset, code_offset, len);
	trace_op!("CodeCopy: {}", len);

	let memory_offset = as_usize_or_fail!(memory_offset);
	let code_offset = as_usize_or_fail!(code_offset);
	let len = as_usize_or_fail!(len);

	try_or_fail!(state.memory.resize_offset(memory_offset, len));
	match state.memory.copy_large(memory_offset, code_offset, len, &state.code) {
		Ok(()) => Control::Continue(1),
		Err(e) => Control::Exit(e.into()),
	}
}

pub fn calldataload(state: &mut Machine) -> Control {
	pop_u256!(state, index);
	trace_op!("CallDataLoad: {}", index);

	let index = as_usize_or_fail!(index);
	let mut load = [0_u8; 32];

	if index < state.data.len() {
		let len = min(32, state.data.len() - index);
		load[0..len].copy_from_slice(&state.data[index..index + len]);
	}

	push!(state, H256::from(load));
	Control::Continue(1)
}

pub fn calldatasize(state: &mut Machine) -> Control {
	let len = U256::from(state.data.len());
	trace_op!("CallDataSize: {}", len);
	push_u256!(state, len);
	Control::Continue(1)
}

pub fn calldatacopy(state: &mut Machine) -> Control {
	pop_u256!(state, memory_offset, data_offset, len);
	trace_op!("CallDataCopy: {}", len);

	let memory_offset = as_usize_or_fail!(memory_offset);
	let data_offset = as_usize_or_fail!(data_offset);
	let len = as_usize_or_fail!(len);

	if len == 0 {
		return Control::Continue(1)
	}

	try_or_fail!(state.memory.resize_offset(memory_offset, len));
	match state.memory.copy_large(memory_offset, data_offset, len, &state.data) {
		Ok(()) => Control::Continue(1),
		Err(e) => Control::Exit(e.into()),
	}
}

pub fn pop(state: &mut Machine) -> Control {
	pop_u256!(state, _val);
	trace_op!("Pop  [@{}]: {}", state.stack.len(), val);
	Control::Continue(1)
}

pub fn mload(state: &mut Machine) -> Control {
	pop_u256!(state, index);
	trace_op!("MLoad: {}", index);
	let index = as_usize_or_fail!(index);
	try_or_fail!(state.memory.resize_offset(index, 32));
	let value = H256::from_slice(&state.memory.get(index, 32)[..]);
	push!(state, value);
	Control::Continue(1)
}

pub fn mstore(state: &mut Machine) -> Control {
	pop_u256!(state, index);
	pop!(state, value);
	trace_op!("MStore: {}, {}", index, value);
	let index = as_usize_or_fail!(index);
	try_or_fail!(state.memory.resize_offset(index, 32));
	match state.memory.set(index, &value[..], Some(32)) {
		Ok(()) => Control::Continue(1),
		Err(e) => Control::Exit(e.into()),
	}
}

pub fn mstore8(state: &mut Machine) -> Control {
	pop_u256!(state, index, value);
	trace_op!("MStore8: {}, {}", index, value);
	let index = as_usize_or_fail!(index);
	try_or_fail!(state.memory.resize_offset(index, 1));
	#[allow(clippy::cast_possible_truncation)]
	let value = (value.low_u32() & 0xff) as u8;
	match state.memory.set(index, &[value], Some(1)) {
		Ok(()) => Control::Continue(1),
		Err(e) => Control::Exit(e.into()),
	}
}

pub fn jump(state: &mut Machine) -> Control {
	pop_u256!(state, dest);
	let dest = as_usize_or_fail!(dest, ExitError::InvalidJump);
	trace_op!("Jump: {}", dest);

	if state.valids.is_valid(dest) {
		Control::Jump(dest)
	} else {
		Control::Exit(ExitError::InvalidJump.into())
	}
}

pub fn jumpi(state: &mut Machine) -> Control {
	pop_u256!(state, dest, value);
	let dest = as_usize_or_fail!(dest, ExitError::InvalidJump);

	if value == U256::zero() {
		trace_op!("JumpI: skipped");
		Control::Continue(1)
	} else {
		trace_op!("JumpI: {}", dest);
		if state.valids.is_valid(dest) {
			Control::Jump(dest)
		} else {
			Control::Exit(ExitError::InvalidJump.into())
		}
	}
}

pub fn pc(state: &mut Machine, position: usize) -> Control {
	trace_op!("PC");
	push_u256!(state, U256::from(position));
	Control::Continue(1)
}

pub fn msize(state: &mut Machine) -> Control {
	trace_op!("MSize");
	push_u256!(state, U256::from(state.memory.effective_len()));
	Control::Continue(1)
}

pub fn push(state: &mut Machine, n: usize, position: usize) -> Control {
	let end = min(position + 1 + n, state.code.len());
	let val = U256::from_big_endian_fast(&state.code[(position + 1)..end]);

	push_u256!(state, val);
	trace_op!("Push [@{}]: {}", state.stack.len() - 1, val);
	Control::Continue(1 + n)
}

pub fn dup(state: &mut Machine, n: usize) -> Control {
	if let Err(e) = state.stack.dup(n - 1) {
		return Control::Exit(e.into());
	};

	trace_op!("Dup{} [@{}]", n, state.stack.len());

	Control::Continue(1)
}

pub fn swap(state: &mut Machine, n: usize) -> Control {
	if let Err(e) = state.stack.swap(n) {
		return Control::Exit(e.into());
	};

	trace_op!("Swap [@0:@{}]", n);
	Control::Continue(1)
}

pub fn ret(state: &mut Machine) -> Control {
	trace_op!("Return");
	pop_u256!(state, start, len);
	let start = as_usize_or_fail!(start);
	let len = as_usize_or_fail!(len);
	try_or_fail!(state.memory.resize_offset(start, len));
	state.return_range = (start, len);
	Control::Exit(ExitSucceed::Returned.into())
}

pub fn revert(state: &mut Machine) -> Control {
	trace_op!("Revert");
	pop_u256!(state, start, len);
	let start = as_usize_or_fail!(start);
	let len = as_usize_or_fail!(len);
	try_or_fail!(state.memory.resize_offset(start, len));
	state.return_range = (start, len);
	Control::Exit(ExitRevert::Reverted.into())
}
