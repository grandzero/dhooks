use candid::{CandidType, Decode, Deserialize, Encode};
pub use ethers_core::{
    abi::{Contract, Token},
    types::{Address, RecoveryMessage, Signature},
};
use ic_cdk::api::call::{call, CallResult};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, DefaultMemoryImpl, StableBTreeMap, Storable};
mod eth_rpc;
mod util;

use std::result;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{borrow::Cow, cell::RefCell};
use std::{rc::Rc, str::FromStr};
use util::to_hex;

use eth_rpc::call_contract; // Add the missing import

type Memory = VirtualMemory<DefaultMemoryImpl>;
const MAX_VALUE_SIZE: u32 = 5000;

#[derive(CandidType, Deserialize)]
struct Hook {
    rpc: String,
    contract_address: String,
}

impl Storable for Hook {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Hook {
    const MAX_SIZE: u32 = MAX_VALUE_SIZE;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static ICP_HOOK: Rc<Contract> = Rc::new(include_abi!("../abi/icphook.json"));
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    static HOOKS: RefCell<StableBTreeMap<u64, Hook, Memory>> = RefCell::new(StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(0)))));

}

#[ic_cdk::query]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

#[ic_cdk::update]
pub async fn get_data_from_evm(network: String, contract_address: String, token_id: u64) -> String {
    let abi = &ICP_HOOK.with(Rc::clone);
    let result = call_contract(
        &network,
        contract_address,
        abi,
        "ownerOf",
        &[Token::Uint(token_id.into())],
    )
    .await;
    match result.get(0) {
        Some(Token::Address(a)) => to_hex(a.as_bytes()),
        _ => panic!("Unexpected result"),
    }
}
