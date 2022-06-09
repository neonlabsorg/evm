//! Tools for tracing runtime events

// use evm::Context;
// use evm::{H160, H256, U256, Stack, Memory, Opcode, Capture, Trap};
// use evm_runtime::{CreateScheme, ExitReason, Transfer};
use solana_program::tracer_api;

use crate::{H160, H256, U256, Context, Opcode, Stack, Memory, Capture, ExitReason, Trap, CreateScheme,Transfer};
use alloc::vec::Vec;

/// Trace event
#[derive(Debug,  Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Event {
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

pub fn send(event: Event){
    let mut message : Vec<u8> = Vec::new();
    bincode::serialize_into(&mut message, &event).unwrap();
    tracer_api::send_trace_message(message.as_slice());
}
