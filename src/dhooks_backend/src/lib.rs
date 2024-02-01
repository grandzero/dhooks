use candid::{CandidType, Decode, Deserialize, Encode};
use ethers_core::abi::Bytes;
pub use ethers_core::{
    abi::{Contract, Token},
    types::{Address, RecoveryMessage, Signature},
};
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, TransformContext,
};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, DefaultMemoryImpl, StableBTreeMap, Storable};
use serde_json::json;
mod eth_rpc;
mod util;

use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{borrow::Cow, cell::RefCell};

use eth_rpc::call_contract; // Add the missing import

type Memory = VirtualMemory<DefaultMemoryImpl>;
const MAX_VALUE_SIZE: u32 = 5000;
const MAX_RESPONSE_BYTES: u64 = 2048;
const HTTP_CYCLES: u128 = 100_000_000;
#[derive(CandidType, Deserialize, Clone)]
struct Hook {
    rpc_url: String,
    contract_address: String,
    callback_url: String,
    interval_sec: u64,
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
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    static HOOKS: RefCell<StableBTreeMap<u64, Hook, Memory>> = RefCell::new(StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(0)))));
}

async fn call_endpoint(result: String) -> Result<String, String> {
    let hook = HOOKS.with(|hook| hook.borrow().get(&0).unwrap().clone());
    let url = hook.callback_url;

    // Serialize the payload to JSON
    let payload = json!({
        "status": 200,
        "message": result,
        "error": ""
    });
    let serialized_payload =
        serde_json::to_string(&payload).expect("Error while encoding JSON payload");

    // Prepare the HTTP request
    let parsed_url = url::Url::parse(&url).expect("Service URL parse error");
    let host = parsed_url
        .host_str()
        .expect("Invalid service URL host")
        .to_string();
    let request_headers = vec![
        HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        },
        HttpHeader {
            name: "Host".to_string(),
            value: format!("{host}:443"),
        },
        HttpHeader {
            name: "Idempotency-Key".to_string(),
            value: "UUID-123456789".to_string(),
        },
    ];

    let request = CanisterHttpRequestArgument {
        url,
        max_response_bytes: Some(MAX_RESPONSE_BYTES),
        method: HttpMethod::POST,
        headers: request_headers,
        body: Some(serialized_payload.into_bytes()),
        transform: Some(TransformContext::from_name("transform".to_string(), vec![])),
    };
    match http_request(request, HTTP_CYCLES).await {
        Ok((_r,)) => Ok("Endpoint called successfully".to_string()),
        Err((r, m)) => panic!("Error in http_request : {:?} {:?}", r, m),
    }
}

// While initializing the canister, we need to set up the periodic task.
// And get the webhook url, rpc url and address to look from user

#[ic_cdk::init]
fn init(timer_interval_secs: u64, rpc_url: String, contract_address: String, callback_url: String) {
    // Create hook
    HOOKS.with(|hook| {
        hook.borrow_mut().insert(
            0,
            Hook {
                rpc_url,
                contract_address,
                callback_url,
                interval_sec: timer_interval_secs,
            },
        )
    });

    // Spawn interval
    let interval = std::time::Duration::from_secs(timer_interval_secs);
    ic_cdk_timers::set_timer_interval(interval, timer_callback);
}

fn timer_callback() {
    ic_cdk::spawn(async {
        COUNTER.with(|counter| counter.fetch_add(1, Ordering::Relaxed));
        get_data_from_evm().await;
        // Process data here
    });
}

#[ic_cdk::query(name = "get_counter")]
fn get_counter() -> u64 {
    COUNTER.with(|counter| counter.load(Ordering::Relaxed))
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    let hook = HOOKS.with(|hook| hook.borrow().get(&0).unwrap().clone());
    init(
        hook.interval_sec,
        hook.rpc_url,
        hook.contract_address,
        hook.callback_url,
    );
}

#[ic_cdk::update]
pub async fn get_data_from_evm() -> String {
    let abi = &ICP_HOOK.with(Rc::clone);
    let hook = HOOKS.with(|hook| hook.borrow().get(&0).unwrap().clone());
    let result = call_contract(
        &hook.rpc_url,
        hook.contract_address,
        abi,
        "icphook",
        &[Token::Bytes(Bytes::from("hello"))],
    )
    .await;

    let result = match result {
        Ok(response) => call_endpoint(response).await,
        Err(_e) => Err("Error while calling endpoint".to_string()),
    };

    if let Ok(_) = result {
        return String::from("Endpoint called successfully");
    } else {
        return String::from("Error while calling endpoint");
    }
}
