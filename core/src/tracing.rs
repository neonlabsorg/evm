use crate::{H160, H256, U256, Context, Opcode, Stack, Memory, Capture, ExitReason, Trap, CreateScheme, Transfer};
use alloc::vec::Vec;


#[derive(Debug,  Clone)]
pub struct CallTrace<'a>{
    /// Called code address
    pub code_address: H160,
    /// Transfer parameters
    pub transfer: &'a Option<Transfer>,
    /// Input data provided to the call
    pub input: &'a Vec<u8>,
    /// Target gas
    pub target_gas: Option<u64>,
    /// Static call flag
    pub is_static: bool,
    /// Runtime context
    pub context: &'a Context,
}

#[derive(Debug,  Clone)]
pub struct CreateTrace<'a>{
    /// Creator address
    pub caller: H160,
    /// Address of the created account
    pub address: H160,
    /// Scheme
    pub scheme: CreateScheme,
    /// Value the created account is endowed with
    pub value: U256,
    /// Init code
    pub init_code: &'a Vec<u8>,
    /// Target Gas
    pub target_gas: Option<u64>,
}

#[derive(Debug,  Clone)]
pub struct SuicideTrace{
    /// Suicided address
    pub address: H160,
    /// Suicided contract heir
    pub target: H160,
    /// Balance before suicide
    pub balance: U256,
}

#[derive(Debug,  Clone)]
pub struct ExitTrace<'a>{
    pub reason: &'a ExitReason,
    pub return_value: &'a Vec<u8>,
}


#[derive(Debug,  Clone)]
pub struct TransactCallTrace<'a>{
    /// Caller account address
    pub caller: H160,
    /// Destination account address
    pub address: H160,
    /// Value transferred to the destination account
    pub value: U256,
    /// Input data provided to the call
    pub data: &'a Vec<u8>,
    /// Gas Limit
    pub gas_limit: U256,
}

#[derive(Debug,  Clone)]
pub struct TransactCreateTrace<'a>{
    /// Creator address
    pub caller: H160,
    /// Value the created account is endowed with
    pub value: U256,
    /// Init code
    pub init_code: &'a Vec<u8>,
    /// Gas limit
    pub gas_limit: U256,
    /// Address of the created account
    pub address: H160,
}

#[derive(Debug,  Clone)]
pub struct TransactCreate2Trace<'a>{
    /// Creator address
    pub caller: H160,
    /// Value the created account is endowed with
    pub value: U256,
    /// Init code
    pub init_code: &'a Vec<u8>,
    /// Salt
    pub salt: H256,
    /// Gas limit
    pub gas_limit: U256,
    /// Address of the created account
    pub address: H160,
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

#[derive(Debug, Clone)]
pub struct TransferTrace{
    pub source: H160,
    pub target: H160,
    pub value: U256,
}

#[derive(Debug, Clone)]
pub struct WithDrawTrace{
    pub source: H160,
    pub value: U256,
}

#[derive(Debug, Clone)]
pub struct SetStorageTrace{
    pub address: H160,
    pub key: U256,
    pub value: U256,
}

#[derive(Debug, Clone)]
pub struct IncrementNonceTrace{
    pub address: H160,
}

#[derive(Debug, Clone)]
pub struct SetCodeTrace<'a>{
    pub address: H160,
    pub code: &'a Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct SelfDestructTrace{
    pub address: H160,
}

/// Trace event
#[derive(Debug,  Clone)]
pub enum Event<'a>{
    Call(CallTrace<'a>) ,
    Create(CreateTrace<'a>) ,
    Suicide(SuicideTrace) ,
    Exit(ExitTrace<'a>) ,
    TransactCall(TransactCallTrace<'a>) ,
    TransactCreate(TransactCreateTrace<'a>) ,
    TransactCreate2(TransactCreate2Trace<'a>) ,
    Step(StepTrace<'a>) ,
    StepResult(StepResultTrace<'a>),
    SLoad(SLoadTrace),
    SStore(SStoreTrace),
    Transfer(TransferTrace),
    WithDraw(WithDrawTrace),
    SetStorage(SetStorageTrace),
    IncrementNonce(IncrementNonceTrace),
    SetCode(SetCodeTrace),
    SelfDestruct(SelfDestructTrace),
}
