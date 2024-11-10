// use std::{cell::RefCell, time::Duration};

// use crate::{get_rpc_service_base, get_rpc_service_sepolia};
// use alloy::{
//     eips::BlockNumberOrTag,
//     primitives::{address, U256},
//     providers::{Provider, ProviderBuilder},
//     rpc::types::{Filter, Log},
//     sol,
//     sol_types::SolEvent,
//     transports::icp::IcpConfig,
// };
// use ic_cdk_timers::TimerId;

// const POLL_LIMIT: usize = 3;

// struct State {
//     timer_id: Option<TimerId>,
//     logs: Vec<String>,
//     poll_count: usize,
// }

// impl State {
//     fn default() -> State {
//         State {
//             // Store the id of the IC_CDK timer used for polling the EVM RPC periodically.
//             // This id can be used to cancel the timer before the configured `POLL_LIMIT`
//             // has been reached.
//             timer_id: None,
//             // The logs returned by the EVM are stored here for display in the frontend.
//             logs: Vec::new(),
//             // The number of polls made. Polls finish automatically, once the `POLL_LIMIT`
//             // has been reached. This count is used to create a good interactive UI experience.
//             poll_count: 0,
//         }
//     }
// }

// thread_local! {
//     static STATE: RefCell<State> = RefCell::new(State::default());
//     // A mutable vector to store text data, similar to Motoko's `array: [Text]`
//     static ARRAY: RefCell<Vec<String>> = RefCell::new(Vec::new());
//     static LAST_PROCESSED_BLOCK: RefCell<u64> = RefCell::new(0); // Track the last processed block

// }

// // Codegen from ABI file to interact with the contract.
// sol!(
//     #[allow(missing_docs)]
//     #[sol(abi)]
//     USDC,
//     "src/abi/USDC.json"
// );


// /// Using the ICP poller for Alloy allows smart contract canisters
// /// to watch EVM blockchain changes easily. In this example, the canister
// /// watches for USDC transfer logs.
// #[ic_cdk::update]
// pub async fn watch_usdc_transfer_start() -> Result<String, String> {
//     // Don't start a timer if one is already running
//     STATE.with_borrow(|state| {
//         if state.timer_id.is_some() {
//             return Err("Already watching for logs.".to_string());
//         }
//         Ok(())
//     })?;

//     let rpc_service = get_rpc_service_sepolia();
//     let config = IcpConfig::new(rpc_service).set_max_response_size(100_000);
//     let provider = ProviderBuilder::new().on_icp(config);
    
//     // This callback will be called every time new logs are received
//     let callback = |incoming_logs: Vec<Log>| {
//         STATE.with_borrow_mut(|state| {
//             for log in incoming_logs.iter() {
//                 let transfer: Log<USDC::Transfer> = log.log_decode().unwrap();
//                 let USDC::Transfer { from, to, value } = transfer.data();
//                 if *to == address!("7f7346b12Ec7d7aa8fAD8Bc5E0a914919368a139") {
//                     let from_fmt = format!("{}", &from);
//                     let to_fmt = format!("{}", &to);
//                     state
//                         .logs
//                         .push(format!("{from_fmt} -> {to_fmt}, value: {value:?}"));
//                     // Additionally, append the data to ARRAY
//                     ARRAY.with(|array| {
//                         array
//                             .borrow_mut()
//                             .push(format!("{from_fmt} -> {to_fmt}, value: {value:?}"))
//                     });

//                 }
//             }

//             state.poll_count += 1;
//             if state.poll_count >= POLL_LIMIT {
//                 state.timer_id.take();
//             }
//         })
//     };

//     // Clear the logs and poll count when starting a new watch
//     STATE.with_borrow_mut(|state| {
//         state.logs.clear();
//         state.poll_count = 0;
//     });

//     let usdt_token_address = address!("1c7d4b196cb0c7b01d743fbc6116a902379c7238");


//     let filter = Filter::new()
//         .address(usdt_token_address)
//         // By specifying an `event` or `event_signature` we listen for a specific event of the
//         // contract. In this case the `Transfer(address,address,uint256)` event.
//         .event(USDC::Transfer::SIGNATURE)
//         .from_block(BlockNumberOrTag::Latest);

//     // Initialize the poller and start watching
//     // `with_limit` (optional) is used to limit the number of times to poll, defaults to 3
//     // `with_poll_interval` (optional) is used to set the interval between polls, defaults to 7 seconds
//     let poller = provider.watch_logs(&filter).await.unwrap();
//     let timer_id = poller
//         .with_limit(Some(POLL_LIMIT))
//         .with_poll_interval(Duration::from_secs(10))
//         .start(callback)
//         .unwrap();

//     // Save timer id to be able to stop watch before completion
//     STATE.with_borrow_mut(|state| {
//         state.timer_id = Some(timer_id);
//     });

//     Ok(format!("Watching for logs, polling {} times.", POLL_LIMIT))
// }

// /// Stop the watch before it reaches completion
// #[ic_cdk::update]
// async fn watch_usdc_transfer_stop() -> Result<String, String> {
//     STATE.with_borrow_mut(|state| {
//         if let Some(timer_id) = state.timer_id.take() {
//             ic_cdk_timers::clear_timer(timer_id);
//             Ok(())
//         } else {
//             Err("No timer to clear.".to_string())
//         }
//     })?;

//     Ok("Watching for logs stopped.".to_string())
// }

// /// Returns a boolean that is `true` when watching and `false` otherwise.
// #[ic_cdk::query]
// async fn watch_usdc_transfer_is_polling() -> Result<bool, String> {
//     STATE.with_borrow(|state| Ok(state.timer_id.is_some()))
// }

// /// Returns the number of polls made. Polls finish automatically, once the `POLL_LIMIT`
// /// has been reached. This count is used to create a good interactive UI experience.
// #[ic_cdk::query]
// async fn watch_usdc_transfer_poll_count() -> Result<usize, String> {
//     STATE.with_borrow(|state| Ok(state.poll_count))
// }

// /// Returns the list of logs returned by the watch. Gets reset on each start.
// #[ic_cdk::query]
// async fn watch_usdc_transfer_get() -> Result<Vec<String>, String> {
//     STATE.with_borrow(|state| Ok(state.logs.iter().map(|log| format!("{log:?}")).collect()))
// }

// #[ic_cdk::query]
// fn data_history() -> Vec<String> {
//     // Access the ARRAY and return a clone of its data
//     ARRAY.with(|array| array.borrow().clone())
// }
