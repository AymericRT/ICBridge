# `IC Bridge`

IC Bridge, is a simple proof-of-concept cross-chain bridge built on ICP, written Rust, that facilitates the bridging of USDC Testnet Tokens from Sepolia to Base Sepolia and vice versa. It heavily relies on the [IC-Alloy](https://ic-alloy.dev) Rust Library to abstract a lot of complexities involved with interacting with the EVM RPC and building an EIP-1559 transaction .

This project is the 1st prize winner for the [2024 Chain Fusion Hacker House Devcon Bankok](https://github.com/ICP-Hacker-House/Devcon_BKK?tab=readme-ov-file#2024-chain-fusion-hacker-house-devcon-bangkok), specifically the [Chain Fusion Track](https://github.com/ICP-Hacker-House/Devcon_BKK?tab=readme-ov-file#2-chain-fusion--total-prize-pool-of-usd-5k).

### Outline
- IC Bridge POC Architecture
- The use of Chain Fusion Technology and Timers in this project
- Quick Codebase walk through and explanation
- Deploy the project Locally
- Deploy the project on Mainnet

## IC Bridge POC Architecture
A SENDER will transfer a small amount of USDC to the Canister's EVM address in the **Sepolia** chain. The canister will pick up the transfer event and send the same corresponding amount to the SENDER's address on the **Base Sepolia** chain, the same logic applies in the other direction, from Base Sepolia to Sepolia. Hence achieving a simple POC for a cross-evm bridge.

![IC Bridge POC](IC-Bridge-POC.png)

This approach demonstrates a simple yet effective proof of concept for a cross-EVM bridge by utilizing a liquidity provisioning model. The bridge maintains dedicated pools of USDC on both networks, enabling instant and seamless token transfers without the need for actual cross-chain token movements.

## The use of Chain Fusion Technology and Timers in this project
Chain Fusion Technology, uses [Threshold ECDSA](https://internetcomputer.org/docs/current/developer-docs/smart-contracts/signatures/t-ecdsa), which allows canisters to have an ECSDA Public Key as well as sign messages. This imples that they are able to create valid EIP-1559 transactions.

Utilizing [HTTP Outcalls](https://internetcomputer.org/https-outcalls), canisters are able to perform read and write operations securely to any EVM Chain, and most importantly, they can do so in an decentralized manner.

Moreover, with [Timers](https://internetcomputer.org/docs/current/developer-docs/smart-contracts/advanced-features/periodic-tasks/#timers), canisters can autonomously read transactions on an EVM Chain. What we mean by this is that without user intervention, we can set the Canister to periodically call the read function to detect transactions, therefore achieving autonomy.

Once the Canister picks up a transfer event targeted towards the Canister's EVM Address on one chain, it will trigger the transfer function directed towards the SENDER address on the other chain.

This project combines the three principles mentioned above: [Threshold ECDSA](https://internetcomputer.org/docs/current/developer-docs/smart-contracts/signatures/t-ecdsa), [HTTP Outcalls](https://internetcomputer.org/https-outcalls) and [Timers](https://internetcomputer.org/docs/current/developer-docs/smart-contracts/advanced-features/periodic-tasks/#timers) to create a cross-evm bridge POC in the Rust.

## Code Walk Through
The important codes are located at [lib.rs](src/ICPBridge_backend/src/lib.rs). We used [IC-Alloy's Toolkit Template](https://github.com/ic-alloy/ic-alloy-toolkit) as heavy inspiration behind our codes.

