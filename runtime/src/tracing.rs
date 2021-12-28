//! Allows to listen to runtime events.

use crate::{Context, Opcode, Stack, Memory, Capture, ExitReason, Trap};
use crate::{H160, U256};

environmental::environmental!(listener: dyn EventListener + 'static);

pub trait EventListener {
    fn event(
        &mut self,
        event: Event
    );
}

#[derive(Debug, Copy, Clone)]
pub enum Event<'a> {
    Step {
        context: &'a Context,
        opcode: Opcode,
        position: &'a Result<usize, ExitReason>,
        stack: &'a Stack,
        memory: &'a Memory
    },
    StepResult {
        result: &'a Result<(), Capture<ExitReason, Trap>>,
        return_value: &'a [u8],
        stack: &'a Stack,
        memory: &'a Memory
    },
    SLoad {
        address: H160,
        index: U256,
        value: U256
    },
    SStore {
        address: H160,
        index: U256,
        value: U256
    },
}

/// Run closure with provided listener.
pub fn using<R, F: FnOnce() -> R>(
    new: &mut (dyn EventListener + 'static),
    f: F
) -> R {
    listener::using(new, f)
}

pub(crate) fn with<F: FnOnce(&mut (dyn EventListener + 'static))>(f: F) {
       listener::with(f);
}

