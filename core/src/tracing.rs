use crate::{H160, U256, Context, Opcode, Stack, Memory, Capture, ExitReason, Trap};
use alloc::vec::Vec;

environmental::environmental!(listener: dyn EventListener + 'static);

pub trait EventListener {
    fn event(&mut self, event: Event);
}

#[derive(Debug,  Clone)]
pub struct StepTrace<'a>{
    pub context: &'a Context,
    pub opcode: Opcode,
    pub position: &'a Result<usize, ExitReason>,
    pub stack: &'a Stack,
    pub memory: &'a Memory,
}

#[derive(Debug,  Clone)]
pub struct StepResultTrace<'a>{
    pub result: &'a Result<(), Capture<ExitReason, Trap>>,
    pub return_value: &'a Vec<u8>,
    pub stack: &'a Stack,
    pub memory: &'a Memory,
}

#[derive(Debug,  Clone)]
pub struct SLoadTrace{
    pub address: H160,
    pub index: U256,
    pub value: U256
}

#[derive(Debug,  Clone)]
pub struct SStoreTrace {
    pub address: H160,
    pub index: U256,
    pub value: U256
}

/// Trace event
#[derive(Debug,  Clone)]
pub enum Event<'a>{
    Step(StepTrace<'a>) ,
    StepResult(StepResultTrace<'a>),
    SLoad(SLoadTrace),
    SStore(SStoreTrace),
}


pub fn with<F: FnOnce(&mut (dyn EventListener + 'static))>(f: F) {
    listener::with(f);
}

pub fn using<R, F: FnOnce() -> R>(new: &mut (dyn EventListener + 'static), f: F) -> R {
    listener::using(new, f)
}
