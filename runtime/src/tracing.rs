//! Tools for tracing runtime events

// use evm::Context;
// use evm::{H160, H256, U256, Stack, Memory, Opcode, Capture, Trap};
// use evm_runtime::{CreateScheme, ExitReason, Transfer};

use crate::{H160, H256, U256, Context, Opcode, Stack, Memory, Capture, ExitReason, Trap, CreateScheme,Transfer};
use alloc::vec::Vec;
// use solana_program::{tracer_api, compute_meter_remaining, compute_meter_set_remaining};

/// Trace event
#[derive(Debug,  Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Event{
    /// Call event
    Call {
        /// Called code address
        code_address: H160,
        /// Transfer parameters
        transfer:  Option<Transfer>,
        /// Input data provided to the call
        #[serde(with = "serde_bytes")]
        input: Vec<u8>,
        /// Target gas
        target_gas: Option<u64>,
        /// Static call flag
        is_static: bool,
        /// Runtime context
        context: Context,
    },
    /// Create event
    Create {
        /// Creator address
        caller: H160,
        /// Address of the created account
        address: H160,
        /// Scheme
        scheme: CreateScheme,
        /// Value the created account is endowed with
        value: U256,
        /// Init code
        #[serde(with = "serde_bytes")]
        init_code: Vec<u8>,
        /// Target Gas
        target_gas: Option<u64>,
    },
    /// Suicide event
    Suicide {
        /// Suicided address
        address: H160,
        /// Suicided contract heir
        target: H160,
        /// Balance before suicide
        balance: U256,
    },
    /// Exit event
    Exit {
        /// Exit reason
        reason: ExitReason,
        /// Return value
        #[serde(with = "serde_bytes")]
        return_value: Vec<u8>,
    },
    /// Transactional Call event
    TransactCall {
        /// Caller account address
        caller: H160,
        /// Destination account address
        address: H160,
        /// Value transferred to the destination account
        value: U256,
        /// Input data provided to the call
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
        /// Gas Limit
        gas_limit: U256,
    },
    /// Transactional Create event
    TransactCreate {
        /// Creator address
        caller: H160,
        /// Value the created account is endowed with
        value: U256,
        /// Init code
        #[serde(with = "serde_bytes")]
        init_code: Vec<u8>,
        /// Gas limit
        gas_limit: U256,
        /// Address of the created account
        address: H160,
    },
    /// Transactional Create2 event
    TransactCreate2 {
        /// Creator address
        caller: H160,
        /// Value the created account is endowed with
        value: U256,
        /// Init code
        #[serde(with = "serde_bytes")]
        init_code: Vec<u8>,
        /// Salt
        salt: H256,
        /// Gas limit
        gas_limit: U256,
        /// Address of the created account
        address: H160,
    },
    Step {
        context: Context,
        opcode: Opcode,
        position: Result<usize, ExitReason>,
        stack: Stack,
        memory: Memory
    },
    StepResult {
        result: Result<(), Capture<ExitReason, Trap>>,
        #[serde(with = "serde_bytes")]
        return_value: Vec<u8>,
        stack: Stack,
        memory: Memory
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

/// EVM stack.
#[derive(Clone, Debug)]
pub struct StackOnStack{
    pub data: u64,/// &[U256],
    pub data_len: usize,
    pub limit: usize,
}

#[derive(Clone, Debug)]
pub struct MemoryOnStack {
    pub data: u64, // &'a[u8],
    pub data_len: usize,
    pub effective_len: usize,
    pub limit: usize,
}


#[derive(Debug,  Clone)]
pub struct CallTrace{
    /// Called code address
    pub code_address: H160,
    /// Transfer parameters
    pub transfer:  Option<Transfer>,
    /// Input data provided to the call
    pub input: u64,
    pub input_len: usize,
    /// Target gas
    pub target_gas: Option<u64>,
    /// Static call flag
    pub is_static: bool,
    /// Runtime context
    pub context: Context,
}

#[derive(Debug,  Clone)]
pub struct CreateTrace{
    /// Creator address
    pub caller: H160,
    /// Address of the created account
    pub address: H160,
    /// Scheme
    pub scheme: CreateScheme,
    /// Value the created account is endowed with
    pub value: U256,
    /// Init code
    pub init_code: u64,
    pub init_code_len: usize,
    /// Target Gas
    pub target_gas: Option<u64>,
}

#[derive(Debug,  Clone)]
pub struct ExitTrace{
    pub reason: ExitReason,
    pub return_value: u64,
    pub return_value_len: usize,
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
pub struct TransactCallTrace{
    /// Caller account address
    pub caller: H160,
    /// Destination account address
    pub address: H160,
    /// Value transferred to the destination account
    pub value: U256,
    /// Input data provided to the call
    pub data: u64,
    pub data_len: usize,
    /// Gas Limit
    pub gas_limit: U256,
}


#[derive(Debug,  Clone)]
pub struct TransactCreateTrace{
    /// Creator address
    pub caller: H160,
    /// Value the created account is endowed with
    pub value: U256,
    /// Init code
    pub init_code: u64,
    pub init_code_len: usize,
    /// Gas limit
    pub gas_limit: U256,
    /// Address of the created account
    pub address: H160,
}

#[derive(Debug,  Clone)]
pub struct TransactCreate2Trace{
    /// Creator address
    pub caller: H160,
    /// Value the created account is endowed with
    pub value: U256,
    /// Init code
    pub init_code: u64,
    pub init_code_len: usize,
    /// Salt
    pub salt: H256,
    /// Gas limit
    pub gas_limit: U256,
    /// Address of the created account
    pub address: H160,
}

#[derive(Debug,  Clone)]
pub struct StepTrace{
    pub context: Context,
    pub opcode: Opcode,
    pub position: Result<usize, ExitReason>,
    pub stack: StackOnStack,
    pub memory: MemoryOnStack,
    pub vec: Vec<u8>,
}

#[derive(Debug,  Clone)]
pub struct StepResultTrace{
    pub result: Result<(), Capture<ExitReason, Trap>>,
    pub return_value: u64,
    pub return_value_len: usize,
    pub stack: StackOnStack,
    pub memory: MemoryOnStack,
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
pub enum EventOnStack{
    Call(CallTrace) ,
    Create(CreateTrace) ,
    Suicide(SuicideTrace) ,
    Exit(ExitTrace) ,
    TransactCall(TransactCallTrace) ,
    TransactCreate(TransactCreateTrace) ,
    TransactCreate2(TransactCreate2Trace) ,
    Step(StepTrace) ,
    StepResult(StepResultTrace),
    SLoad(SLoadTrace),
    SStore(SStoreTrace),
}
