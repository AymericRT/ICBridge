use alloy::{
    providers::{Provider, ProviderBuilder},
    transports::icp::IcpConfig,
};

use crate::get_rpc_service_sepolia;

/// Request the latest block from Eth Sepolia.
#[ic_cdk::update]
pub async fn get_latest_block() -> Result<u64, String> {
    let rpc_service = get_rpc_service_sepolia();
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new().on_icp(config);
    let result = provider.get_block_number().await;

    match result {
        Ok(block) => Ok(block),
        Err(e) => Err(e.to_string()),
    }
}
