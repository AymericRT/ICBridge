mod service;

use alloy::{
    eips::BlockNumberOrTag,
    primitives::{address},
    providers::{Provider, ProviderBuilder},
    rpc::types::{Filter, Log},
    signers::icp::IcpSigner,
    sol,
    sol_types::SolEvent,
    transports::icp::{IcpConfig, RpcApi, RpcService},
};

use ic_cdk::export_candid;
use ic_cdk_timers::TimerId;
use service::transfer_usdc::transfer_usdc;
use service::transfer_usdc_base::transfer_usdc_base;
use std::{cell::RefCell, time::Duration};

// Constants
const POLL_LIMIT: usize = 3;
const N: Duration = Duration::from_secs(15);

// Timer and asynchronous functions
#[ic_cdk::update]
fn start_timer_sepolia() {
    ic_cdk_timers::set_timer_interval(N, || ic_cdk::spawn(async_function_sepolia()));
}

#[ic_cdk::update]
fn start_timer_base() {
    ic_cdk_timers::set_timer_interval(N, || ic_cdk::spawn(async_transfer_base()));
}

// Function to call the watch_usdc_transfer_sepolia
async fn async_function_sepolia() {
    let _ = watch_usdc_transfer_start_sepolia().await.unwrap();
}

// Function to call the watch_usdc_transfer_base
async fn async_transfer_base() {
    let _ = watch_usdc_transfer_start_base().await.unwrap();
}

// RPC services for EVM proxies
fn get_rpc_service_sepolia() -> RpcService {
    RpcService::Custom(RpcApi {
        url: "https://ic-alloy-evm-rpc-proxy.kristofer-977.workers.dev/eth-sepolia".to_string(),
        headers: None,
    })
}

fn get_rpc_service_basesepolia() -> RpcService {
    RpcService::Custom(RpcApi {
        url: "https://ic-alloy-evm-rpc-proxy.kristofer-977.workers.dev/base-sepolia".to_string(),
        headers: None,
    })
}

// ECDSA key name retrieval
fn get_ecdsa_key_name() -> String {
    #[allow(clippy::option_env_unwrap)]
    let dfx_network = option_env!("DFX_NETWORK").unwrap();
    match dfx_network {
        "local" => "dfx_test_key".to_string(),
        "ic" => "key_1".to_string(),
        _ => panic!("Unsupported network."),
    }
}

async fn create_icp_signer() -> IcpSigner {
    let ecdsa_key_name = get_ecdsa_key_name();
    IcpSigner::new(vec![], &ecdsa_key_name, None).await.unwrap()
}

// State management and thread-local storage
struct State {
    timer_id: Option<TimerId>,
    logs: Vec<String>,
    poll_count: usize,
}

impl State {
    fn default() -> State {
        State {
            timer_id: None,
            logs: Vec::new(),
            poll_count: 0,
        }
    }
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
    static ARRAY: RefCell<Vec<String>> = RefCell::new(Vec::new());
    static LAST_PROCESSED_BLOCK: RefCell<u64> = RefCell::new(0);
}


// Solidity ABI code generation for USDC contract
sol!(
    #[allow(missing_docs)]
    #[sol(abi)]
    USDC,
    "src/abi/USDC.json"
);


// Watch for USDC transfer logs at Sepolia
#[ic_cdk::update]
pub async fn watch_usdc_transfer_start_sepolia() -> Result<String, String> {
    STATE.with_borrow(|state| {
        if state.timer_id.is_some() {
            return Err("Already watching for logs.".to_string());
        }
        Ok(())
    })?;

    let rpc_service = get_rpc_service_sepolia();
    let config = IcpConfig::new(rpc_service).set_max_response_size(100_000);
    let provider = ProviderBuilder::new().on_icp(config);

    let callback = |incoming_logs: Vec<Log>| {
        STATE.with_borrow_mut(|state| {
            for log in incoming_logs.iter() {
                let transfer: Log<USDC::Transfer> = log.log_decode().unwrap();
                let USDC::Transfer { from, to, value } = transfer.data();

                let from_address = *from;
                let transfer_value = *value;

                let from_fmt = format!("{}", &from);
                let to_fmt = format!("{}", &to);

                let to_address = address!("7f7346b12Ec7d7aa8fAD8Bc5E0a914919368a139");
                if *to == to_address {
                    ARRAY.with(|array| {
                        array
                            .borrow_mut()
                            .push(format!("{from_fmt} -> {to_fmt}, value: {value:?}"))
                    });
                    // Invoke transfer_usdc function if the address matches
                    ic_cdk::spawn(async move {
                        if let Err(e) = transfer_usdc_base(from_address, transfer_value).await {
                            ic_cdk::println!("Error in transfer_usdc: {:?}", e);
                        }
                    });
                }
            }
            state.poll_count += 1;
            if state.poll_count >= POLL_LIMIT {
                state.timer_id.take();
            }
        })
    };

    STATE.with_borrow_mut(|state| {
        state.logs.clear();
        state.poll_count = 0;
    });

    let usdt_token_address = address!("1c7d4b196cb0c7b01d743fbc6116a902379c7238");

    let filter = Filter::new()
        .address(usdt_token_address)
        .event(USDC::Transfer::SIGNATURE)
        .from_block(BlockNumberOrTag::Latest);

    let poller = provider.watch_logs(&filter).await.unwrap();

    let timer_id = poller
        .with_limit(Some(POLL_LIMIT))
        .with_poll_interval(Duration::from_secs(10))
        .start(callback)
        .unwrap();

    STATE.with_borrow_mut(|state| {
        state.timer_id = Some(timer_id);
    });

    Ok(format!("Watching for logs, polling {} times.", POLL_LIMIT))
}

// Watch for USDC transfer logs at Base
#[ic_cdk::update]
pub async fn watch_usdc_transfer_start_base() -> Result<String, String> {
    STATE.with_borrow(|state| {
        if state.timer_id.is_some() {
            return Err("Already watching for logs.".to_string());
        }
        Ok(())
    })?;

    let rpc_service = get_rpc_service_basesepolia();
    let config = IcpConfig::new(rpc_service).set_max_response_size(100_000);
    let provider = ProviderBuilder::new().on_icp(config);

    let callback = |incoming_logs: Vec<Log>| {
        STATE.with_borrow_mut(|state| {
            for log in incoming_logs.iter() {
                let transfer: Log<USDC::Transfer> = log.log_decode().unwrap();
                let USDC::Transfer { from, to, value } = transfer.data();

                let from_address = *from;
                let transfer_value = *value;

                let from_fmt = format!("{}", &from);
                let to_fmt = format!("{}", &to);

                let to_address = address!("7f7346b12Ec7d7aa8fAD8Bc5E0a914919368a139");
                if *to == to_address {
                    ARRAY.with(|array| {
                        array
                            .borrow_mut()
                            .push(format!("{from_fmt} -> {to_fmt}, value: {value:?}"))
                    });
                    // Invoke transfer_usdc function if the address matches
                    ic_cdk::spawn(async move {
                        if let Err(e) = transfer_usdc(from_address, transfer_value).await {
                            ic_cdk::println!("Error in transfer_usdc: {:?}", e);
                        }
                    });
                }
            }
            state.poll_count += 1;
            if state.poll_count >= POLL_LIMIT {
                state.timer_id.take();
            }
        })
    };

    STATE.with_borrow_mut(|state| {
        state.logs.clear();
        state.poll_count = 0;
    });

    let usdt_token_address = address!("036CbD53842c5426634e7929541eC2318f3dCF7e");

    let filter = Filter::new()
        .address(usdt_token_address)
        .event(USDC::Transfer::SIGNATURE)
        .from_block(BlockNumberOrTag::Latest);

    let poller = provider.watch_logs(&filter).await.unwrap();

    let timer_id = poller
        .with_limit(Some(POLL_LIMIT))
        .with_poll_interval(Duration::from_secs(10))
        .start(callback)
        .unwrap();

    STATE.with_borrow_mut(|state| {
        state.timer_id = Some(timer_id);
    });

    Ok(format!("Watching for logs, polling {} times.", POLL_LIMIT))
}

// Check if the watch_usdc_transfer works
#[ic_cdk::query]
async fn watch_usdc_transfer_is_polling() -> Result<bool, String> {
    STATE.with_borrow(|state| Ok(state.timer_id.is_some()))
}

// Query the read history
#[ic_cdk::query]
fn data_history() -> Vec<String> {
    ARRAY.with(|array| array.borrow().clone())
}



export_candid!();
