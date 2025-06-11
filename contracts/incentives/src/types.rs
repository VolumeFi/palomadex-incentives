use std::fmt::{Display, Formatter, Result};
use std::hash::{Hash, Hasher};

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, CustomMsg, Decimal256, Env, StdError, StdResult, Uint128};

use crate::asset::Asset;
use crate::asset::AssetInfo;
use crate::asset::PairInfo;
use crate::constants::{EPOCHS_START, EPOCH_LENGTH, MAX_PERIODS};

#[cw_serde]
pub struct InputSchedule {
    pub reward: Asset,
    pub duration_periods: u64,
}

#[cw_serde]
pub struct IncentivizationFeeInfo {
    /// Fee receiver can be either a contract or a wallet.
    pub fee_receiver: Addr,
    /// To make things easier we avoid CW20 fee tokens
    pub fee: Coin,
}

#[derive(Eq)]
#[cw_serde]
pub enum PairType {
    /// XYK pair type
    Xyk {},
    /// Stable pair type
    Stable {},
    /// Custom pair type
    Custom(String),
}

/// Returns a raw encoded string representing the name of each pool type
impl Display for PairType {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        match self {
            PairType::Xyk {} => fmt.write_str("xyk"),
            PairType::Stable {} => fmt.write_str("stable"),
            PairType::Custom(pair_type) => fmt.write_str(format!("custom-{}", pair_type).as_str()),
        }
    }
}

#[cw_serde]
/// Cw20 hook message template
pub enum Cw20Msg {
    Deposit {
        recipient: Option<String>,
    },
    /// Besides this enum variant is redundant we keep this for backward compatibility with old pair contracts
    DepositFor(String),
}

/// This structure describes the parameters used for creating a request for a change of contract ownership.
#[cw_serde]
pub struct OwnershipProposal {
    /// The newly proposed contract owner
    pub owner: Addr,
    /// Time until the proposal to change ownership expires
    pub ttl: u64,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum PairQueryMsg {
    /// Returns information about a pair in an object of type [`super::asset::PairInfo`].
    #[returns(PairInfo)]
    Pair {},
    #[returns(SimulationResponse)]
    Simulation {
        offer_asset: Asset,
        ask_asset_info: Option<AssetInfo>,
    },
    #[returns(ReverseSimulationResponse)]
    ReverseSimulation {
        offer_asset_info: Option<AssetInfo>,
        ask_asset: Asset,
    },
}

#[cw_serde]
#[derive(Eq)]
/// This enum is a tiny wrapper over [`AssetInfo`] to differentiate between internal and external rewards.
/// External rewards always have a next_update_ts field which is used to update reward per second (or disable them).
pub enum RewardType {
    /// Internal rewards aka PDEX emissions don't have next_update_ts field and they are paid out.
    Int(AssetInfo),
    /// External rewards always have corresponding schedules. Reward is paid out from Generator contract balance.
    Ext {
        info: AssetInfo,
        /// Time when next schedule should start
        next_update_ts: u64,
    },
}

impl RewardType {
    pub fn is_external(&self) -> bool {
        matches!(&self, RewardType::Ext { .. })
    }

    pub fn asset_info(&self) -> &AssetInfo {
        match &self {
            RewardType::Int(info) | RewardType::Ext { info, .. } => info,
        }
    }

    pub fn matches(&self, other: &Self) -> bool {
        match (&self, other) {
            (RewardType::Int(..), RewardType::Int(..)) => true,
            (RewardType::Ext { info: info1, .. }, RewardType::Ext { info: info2, .. }) => {
                info1 == info2
            }
            _ => false,
        }
    }
}

impl Hash for RewardType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // We ignore next_update_ts field to have the same hash for the same external reward token
        match self {
            RewardType::Int(info) => {
                state.write_u8(0);
                info.hash(state);
            }
            RewardType::Ext { info, .. } => {
                state.write_u8(1);
                info.hash(state);
            }
        }
    }

    #[cfg(not(tarpaulin_include))]
    fn hash_slice<H: Hasher>(data: &[Self], state: &mut H)
    where
        Self: Sized,
    {
        for d in data {
            d.hash(state);
        }
    }
}

#[cw_serde]
pub struct Config {
    /// Address allowed to change contract parameters
    pub owner: Addr,
    /// The Factory address
    pub factory: Addr,
    /// Contract address which can only set active generators and their alloc points
    pub generator_controller: Option<Addr>,
    /// [`AssetInfo`] of the PDEX token
    pub pdex_token: AssetInfo,
    /// Total amount of PDEX rewards per second
    pub pdex_per_second: Uint128,
    /// Total allocation points. Must be the sum of all allocation points in all active generators
    pub total_alloc_points: Uint128,
    /// Defines native fee along with fee receiver.
    /// Fee is paid on adding NEW external reward to a specific pool
    pub incentivization_fee_info: Option<IncentivizationFeeInfo>,
}

#[cw_serde]
pub struct FactoryConfig {
    /// Address allowed to change contract parameters
    pub owner: Addr,
    /// CW20 token contract code identifier
    pub token_code_id: u64,
    /// Generator contract address
    pub generator_address: Option<Addr>,
    /// Contract address to send governance fees to (the Maker contract)
    pub fee_address: Option<Addr>,
    /// CW1 whitelist contract code id used to store 3rd party generator staking rewards
    pub whitelist_code_id: u64,
    /// The address of the contract that contains the coins with their precision
    pub coin_registry_address: Addr,
}

#[cw_serde]
pub struct FeeInfoResponse {
    /// Contract address to send governance fees to
    pub fee_address: Option<Addr>,
    /// Total amount of fees (in bps) charged on a swap
    pub total_fee_bps: u16,
    /// Amount of fees (in bps) sent to the Maker contract
    pub maker_fee_bps: u16,
}

#[cw_serde]
pub struct PairsResponse {
    /// Arrays of structs containing information about multiple pairs
    pub pairs: Vec<PairInfo>,
}

/// This structure holds the parameters that are returned from a swap simulation response
#[cw_serde]
pub struct SimulationResponse {
    /// The amount of ask assets returned by the swap
    pub return_amount: Uint128,
    /// The spread used in the swap operation
    pub spread_amount: Uint128,
    /// The amount of fees charged by the transaction
    pub commission_amount: Uint128,
}

#[cw_serde]
pub struct ReverseSimulationResponse {
    /// The amount of offer assets returned by the reverse swap
    pub offer_amount: Uint128,
    /// The spread used in the swap operation
    pub spread_amount: Uint128,
    /// The amount of fees charged by the transaction
    pub commission_amount: Uint128,
}

#[cw_serde]
pub struct ScheduleResponse {
    pub rps: Decimal256,
    pub start_ts: u64,
    pub end_ts: u64,
}

#[cw_serde]
pub struct RewardInfo {
    /// Defines [`AssetInfo`] of reward token as well as its type: protocol or external.
    pub reward: RewardType,
    /// Reward tokens per second for the whole pool
    pub rps: Decimal256,
    /// Last checkpointed reward per LP token
    pub index: Decimal256,
    /// Orphaned rewards might appear between the time when pool
    /// gets incentivized and the time when first user stakes
    pub orphaned: Decimal256,
}

#[cw_serde]
pub struct PoolInfoResponse {
    /// Total amount of LP tokens staked in this pool
    pub total_lp: Uint128,
    /// Vector contains reward info for each reward token
    pub rewards: Vec<RewardInfo>,
    /// Last time when reward indexes were updated
    pub last_update_ts: u64,
}

#[cw_serde]
pub struct IncentivesSchedule {
    /// Schedule start time (matches with epoch start time i.e. on Monday)
    pub next_epoch_start_ts: u64,
    /// Schedule end time (matches with epoch start time i.e. on Monday)
    pub end_ts: u64,
    /// Reward asset info
    pub reward_info: AssetInfo,
    /// Reward per second for the whole schedule
    pub rps: Decimal256,
}

impl IncentivesSchedule {
    /// Creates a new incentives schedule starting now and lasting for the specified number of periods.
    pub fn from_input(env: &Env, input: &InputSchedule) -> StdResult<Self> {
        if input.duration_periods > MAX_PERIODS || input.duration_periods == 0 {
            return Err(StdError::generic_err(format!(
                "Duration must be more 0 and less than or equal to {MAX_PERIODS}",
            )));
        }

        let block_ts = env.block.time.seconds();

        let rem = block_ts % EPOCHS_START;
        let next_epoch_start_ts = if rem % EPOCH_LENGTH == 0 {
            // Hit at the beginning of the current epoch
            block_ts
        } else {
            // Hit somewhere in the middle.
            // Partially distribute rewards for the current epoch and add input.duration_periods periods more
            EPOCHS_START + (rem / EPOCH_LENGTH + 1) * EPOCH_LENGTH
        };
        let end_ts = next_epoch_start_ts + input.duration_periods * EPOCH_LENGTH;

        let rps = Decimal256::from_ratio(input.reward.amount, end_ts - block_ts);

        if rps < Decimal256::one() {
            return Err(StdError::generic_err(format!(
                "Reward per second must be at least 1 unit but actual is {rps}",
            )));
        }

        Ok(Self {
            next_epoch_start_ts,
            end_ts,
            reward_info: input.reward.info.clone(),
            rps,
        })
    }
}

#[cw_serde]
pub enum PalomaMsg {
    /// Message struct for tokenfactory calls.
    TokenFactoryMsg {
        create_denom: Option<CreateDenomMsg>,
        mint_tokens: Option<MintMsg>,
    },
    SkywayMsg {
        set_erc20_to_denom: SetErc20ToDenom,
    },
}

#[cw_serde]
pub struct CreateDenomMsg {
    pub subdenom: String,
    pub metadata: Metadata,
}

#[cw_serde]
pub struct DenomUnit {
    pub denom: String,
    pub exponent: u32,
    pub aliases: Vec<String>,
}

#[cw_serde]
pub struct Metadata {
    pub description: String,
    pub denom_units: Vec<DenomUnit>,
    pub base: String,
    pub display: String,
    pub name: String,
    pub symbol: String,
}

#[cw_serde]
pub struct MintMsg {
    pub denom: String,
    pub amount: Uint128,
    pub mint_to_address: String,
}

#[cw_serde]
pub struct SetErc20ToDenom {
    pub erc20_address: String,
    pub token_denom: String,
    pub chain_reference_id: String,
}

impl CustomMsg for PalomaMsg {}
