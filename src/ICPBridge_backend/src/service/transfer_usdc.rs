use std::cell::RefCell;

use alloy::{
    network::EthereumWallet,
    primitives::{address, Address, U256},
    providers::{Provider, ProviderBuilder},
    signers::Signer,
    sol,
    transports::icp::IcpConfig,
};

use crate::{create_icp_signer, get_rpc_service_sepolia};

thread_local! {
    static NONCE: RefCell<Option<u64>> = const { RefCell::new(None) };
}

// Codegen from ABI file to interact with the contract.
sol!(
    #[allow(missing_docs, clippy::too_many_arguments)]
    #[sol(rpc)]
    USDC,
    "src/abi/USDC.json"
);

pub async fn transfer_usdc(to: Address, value: U256) -> Result<String, String> {
    // Sanitize input
    let recipient_address = to;
    let transfer_amount = value;

    // Setup signer
    let signer = create_icp_signer().await;
    let address = signer.address();

    // Setup provider
    let wallet = EthereumWallet::from(signer);
    let rpc_service = get_rpc_service_sepolia();
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new()
        .with_gas_estimation()
        .wallet(wallet)
        .on_icp(config);

    // Attempt to get nonce from thread-local storage
    let maybe_nonce = NONCE.with_borrow(|maybe_nonce| {
        // If a nonce exists, the next nonce to use is latest nonce + 1
        maybe_nonce.map(|nonce| nonce + 1)
    });

    // If no nonce exists, get it from the provider
    let nonce = if let Some(nonce) = maybe_nonce {
        nonce
    } else {
        provider.get_transaction_count(address).await.unwrap_or(0)
    };

    let contract = USDC::new(
        address!("1c7d4b196cb0c7b01d743fbc6116a902379c7238"),
        provider.clone(),
    );

    match contract
        .transfer(recipient_address, U256::from(transfer_amount))
        .nonce(nonce)
        .chain_id(11155111)
        .from(address)
        .send()
        .await
    {
        Ok(builder) => {
            let node_hash = *builder.tx_hash();
            let tx_response = provider.get_transaction_by_hash(node_hash).await.unwrap();

            match tx_response {
                Some(tx) => {
                    // The transaction has been mined and included in a block, the nonce
                    // has been consumed. Save it to thread-local storage. Next transaction
                    // for this address will use a nonce that is = this nonce + 1
                    NONCE.with_borrow_mut(|nonce| {
                        *nonce = Some(tx.nonce);
                    });
                    Ok(format!("{:?}", tx))
                }
                None => Err("Could not get transaction.".to_string()),
            }
        }
        Err(e) => Err(format!("{:?}", e)),
    }
}
