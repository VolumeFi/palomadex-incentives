# PalomaDex Incentives Smart Contracts

## Overview
The `palomadex-incentives` repository contains CosmWasm smart contracts written in Rust for the Paloma blockchain, a Cosmos SDK-based chain designed for fast, permissionless cross-chain instruction execution. Developed by VolumeFi, these contracts power the incentives mechanism for PalomaDex, a decentralized exchange (DEX) protocol on the Paloma chain. The contracts facilitate [e.g., automated reward distribution, liquidity incentives, or staking mechanisms] to encourage user participation and enhance liquidity within the PalomaDex ecosystem. Leveraging CosmWasm’s secure and interoperable framework, the contracts enable seamless interaction with Paloma’s cross-chain messaging capabilities and the broader Cosmos ecosystem via the Inter-Blockchain Communication (IBC) protocol.[](https://www.cosmobook.io/cosmobook/readme/paloma)[](https://www.rootdata.com/Projects/detail/Paloma?k=NjQyNQ%253D%253D)

### Features
- **Reward Distribution**: Automates the allocation of tokens to users based on [e.g., liquidity provision, trading volume, or staking duration].
- **Cross-Chain Compatibility**: Integrates with Paloma’s cross-chain messaging to manage incentives across connected blockchains via IBC.[](https://www.cosmobook.io/cosmobook/readme/paloma)
- **Security**: Built with Rust and CosmWasm, preventing common attack vectors like reentrancy through design and enabling robust unit and integration testing.[](https://medium.com/cosmwasm/cosmwasm-for-ctos-f1ffa19cccb8)
- **Modularity**: Contracts are designed for composability, allowing integration with other PalomaDex or Cosmos-based protocols.
- **Transparent State Management**: Utilizes CosmWasm’s storage abstractions for persistent, queryable state on the Paloma blockchain.[](https://docs.palomachain.com/guide/develop/smart-contracts/contracts.html)

## Prerequisites
To develop, test, or deploy these contracts, ensure the following are installed:
- [Rust](https://www.rust-lang.org/) (version 1.64 or higher, with `wasm32` target: `rustup target add wasm32-unknown-unknown`)
- [Docker](https://www.docker.com/) (for reproducible builds with `rust-optimizer`)
- [CosmWasm CLI](https://github.com/CosmWasm/cosmwasm) or [wasmd](https://github.com/CosmWasm/wasmd) (for local testing)
- [Paloma CLI](https://docs.palomachain.com/) (`pigeon` for interacting with the Paloma chain)
- A Paloma-compatible wallet (e.g., [Keplr](https://www.keplr.app/)) for testnet/mainnet interactions
- Access to a Paloma node or RPC endpoint (e.g., `https://lcd.testnet.palomaswap.com/` for testnet)[](https://docs.palomachain.com/guide/develop/quick-start/paloma-py/cw721.html)

## Installation
1. **Clone the Repository**:
   ```bash
   git clone https://github.com/VolumeFi/palomadex-incentives.git
   cd palomadex-incentives
   ```

2. **Install Dependencies**:
   Ensure Rust and the `wasm32` target are installed:
   ```bash
   rustup default stable
   rustup target add wasm32-unknown-unknown
   ```
   Install project dependencies:
   ```bash
   cargo build
   ```

3. **Set Up Environment Variables**:
   Create a `.env` file in the project root:
   ```env
   PALOMA_MNEMONIC="your_wallet_mnemonic_phrase"
   PALOMA_RPC_URL="https://lcd.testnet.palomaswap.com/"
   PALOMA_CHAIN_ID="paloma-testnet-15"
   ```
   Replace `your_wallet_mnemonic_phrase` with your wallet’s mnemonic (never commit to version control) and adjust `PALOMA_RPC_URL` and `PALOMA_CHAIN_ID` for testnet or mainnet as needed.

## Compiling Contracts
Compile the contracts to WebAssembly (Wasm) bytecode using `rust-optimizer` for optimized, reproducible builds:
```bash
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.4
```
This generates optimized `.wasm` files in the `artifacts/` directory, suitable for deployment on Paloma.[](https://docs.palomachain.com/guide/develop/smart-contracts/contracts.html)

## Contract Details
Below is an overview of the key smart contracts in this repository. [Replace placeholders with specific contract details if available.]

### 1. IncentivesDistributor
- **Purpose**: Manages the distribution of rewards to users participating in PalomaDex (e.g., liquidity providers or traders).
- **Key Functions**:
  - `instantiate`: Initializes the contract with parameters like reward token address, distribution schedule, and eligible pools.
  - `execute::distribute_rewards`: Allocates rewards to users based on [e.g., staked amounts or trading volume], triggered by Paloma’s pigeons or scheduled execution.
  - `query::reward_balance`: Returns a user’s pending rewards for a given pool or address.
  - `execute::update_config`: Allows the contract owner to update parameters like reward rates or distribution intervals (governance-protected).
- **State**:
  ```rust
  use cosmwasm_std::Addr;
  use cw_storage_plus::Item;
  pub struct Config {
      pub owner: Addr,
      pub reward_token: Addr,
      pub distribution_interval: u64,
  }
  pub const CONFIG: Item<Config> = Item::new("config");
  ```
- **Dependencies**: Uses `cw_storage_plus` for state management and `cosmwasm_std` for blockchain interactions.[](https://docs.palomachain.com/guide/develop/smart-contracts/contracts.html)

### 2. StakingRewards
- **Purpose**: Facilitates staking of PalomaDex LP tokens to earn additional incentives.
- **Key Functions**:
  - `execute::stake`: Allows users to stake LP tokens to participate in the incentives program.
  - `execute::unstake`: Withdraws staked tokens and claims accumulated rewards.
  - `query::staked_balance`: Retrieves a user’s staked amount and accrued rewards.
- **State**:
  ```rust
  use cosmwasm_std::Addr;
  use cw_storage_plus::Map;
  pub struct Stake {
      pub amount: u128,
      pub last_reward_time: u64,
  }
  pub const STAKES: Map<&Addr, Stake> = Map::new("stakes");
  ```
- **Dependencies**: Integrates with CW20 token standards for handling LP tokens.[](https://docs.palomachain.com/guide/develop/quick-start/paloma-py/cw721.html)

### 3. CrossChainIncentives [Optional]
- **Purpose**: Coordinates incentives across multiple chains via Paloma’s cross-chain messaging and IBC.
- **Key Functions**:
  - `execute::relay_rewards`: Sends reward instructions to a target chain using Paloma’s pigeon validators.
  - `query::cross_chain_status`: Checks the status of cross-chain reward distributions.
- **State**:
  ```rust
  use cosmwasm_std::{Addr, Binary};
  use cw_storage_plus::Map;
  pub struct CrossChainRequest {
      pub target_chain: String,
      pub status: String,
  }
  pub const REQUESTS: Map<&Binary, CrossChainRequest> = Map::new("requests");
  ```
- **Dependencies**: Leverages Paloma’s cross-chain messaging and IBC modules for interoperability.[](https://www.cosmobook.io/cosmobook/readme/paloma)

## Testing
Run the test suite to verify contract functionality:
```bash
cargo test
```
Tests are located in the `tests/` directory and include:
- Unit tests for individual contract functions (e.g., reward calculation logic).
- Integration tests simulating interactions with Paloma’s blockchain environment using `cosmwasm-vm`.[](https://medium.com/cosmwasm/cosmwasm-for-ctos-f1ffa19cccb8)
- Scenarios covering [e.g., reward distribution edge cases, staking/unstaking flows, or cross-chain message relaying].

For local testnet testing:
1. Set up a local Paloma node or use `wasmd` for a CosmWasm-compatible environment:
   ```bash
   docker run -it -p 26657:26657 -p 1317:1317 cosmwasm/wasmd:latest
   ```
2. Deploy and test contracts using Paloma CLI or `paloma.py` SDK.[](https://docs.palomachain.com/guide/develop/quick-start/paloma-py/cw721.html)

## Deployment
To deploy contracts to the Paloma testnet or mainnet:
1. **Store the Contract Code**:
   ```bash
   palomad tx wasm store artifacts/incentives_distributor.wasm --from <wallet> --chain-id paloma-testnet-15 --gas auto
   ```
   Note the `code_id` from the transaction result.
2. **Instantiate the Contract**:
   ```bash
   palomad tx wasm instantiate <code_id> '{"owner":"paloma1...","reward_token":"paloma1...","distribution_interval":86400}' --from <wallet> --chain-id paloma-testnet-15 --gas auto
   ```
   Retrieve the `contract_address` from the transaction logs.
3. **Interact with the Contract**:
   Use `palomad` or `paloma.py` to execute or query the contract:
   ```bash
   palomad query wasm contract-state smart <contract_address> '{"reward_balance":{"address":"paloma1..."}}'
   ```
   For programmatic interaction, use `paloma.py`:
   ```python
   from paloma_sdk.client.lcd import AsyncLCDClient
   from paloma_sdk.core.wasm import MsgExecuteContract
   async def main():
       paloma = AsyncLCDClient(url="https://lcd.testnet.palomaswap.com/", chain_id="paloma-testnet-15")
       wallet = paloma.wallet(MnemonicKey(mnemonic="your_mnemonic"))
       tx = await wallet.create_and_sign_tx(
           CreateTxOptions(msgs=[
               MsgExecuteContract(wallet.key.acc_address, contract_address, {"distribute_rewards": {}})
           ])
       )
       result = await paloma.tx.broadcast(tx)
       print(result)
   asyncio.run(main())
   ```

## Usage
### Interacting with Contracts
- **Command Line**: Use `palomad` CLI to query or execute contract functions (see deployment section).
- **SDK**: Use `paloma.py` for Python-based interactions or `CosmJS` for JavaScript-based frontends.[](https://docs.palomachain.com/guide/develop/quick-start/paloma-py/cw721.html)
- **Cross-Chain**: Leverage Paloma’s pigeon validators to relay incentive-related messages to other chains (e.g., Ethereum, Binance Smart Chain) via IBC or Paloma’s messaging protocol.[](https://www.cosmobook.io/cosmobook/readme/paloma)

### Example Workflow
1. A user stakes tokens in the `StakingRewards` contract.
2. The `IncentivesDistributor` contract periodically calculates and distributes rewards based on staked amounts.
3. For cross-chain incentives, the `CrossChainIncentives` contract sends reward instructions to a target chain, executed by Paloma validators.

## Contributing
Contributions are welcome! To contribute:
1. Fork the repository.
2. Create a feature branch (`git checkout -b feature/your-feature`).
3. Commit changes (`git commit -m 'Add your feature'`).
4. Push to the branch (`git push origin feature/your-feature`).
5. Open a pull request, ensuring tests pass and code adheres to Rust/CosmWasm best practices.

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

## Security
- **Audits**: Contracts should be audited by a reputable firm before mainnet deployment, especially for DeFi protocols.[](https://rudolfolah.com/smart-contracts-for-cosmos-blockchain/)
- **Testing**: Comprehensive unit and integration tests are included to cover critical logic and edge cases.
- **Reentrancy Protection**: CosmWasm’s actor model inherently prevents reentrancy attacks.[](https://medium.com/cosmwasm/cosmwasm-for-ctos-f1ffa19cccb8)
- **Optimization**: Use `rust-optimizer` to produce compact, secure Wasm binaries.[](https://docs.palomachain.com/guide/develop/smart-contracts/contracts.html)

## License
This project is licensed under the [MIT License](LICENSE) [or specify the correct license if known].

## Contact
For support or inquiries, contact the VolumeFi team:
- Website: [https://www.volumefi.com/](https://www.volumefi.com/)
- Discord: [VolumeFi Discord](https://discord.com/invite/volumefi) [replace with actual link]
- Email: [support@volumefi.com](mailto:support@volumefi.com) [replace with actual email]
- Documentation: [Paloma Protocol](https://docs.palomachain.com/)[](https://docs.palomachain.com/guide/develop/quick-start/paloma-py/cw721.html)