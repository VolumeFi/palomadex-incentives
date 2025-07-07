use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cw20::Cw20ReceiveMsg;

#[allow(unused_imports)]
use crate::{
    asset::{Asset, AssetInfo, PairInfo},
    types::{
        Config, FeeInfoResponse, IncentivizationFeeInfo, InputSchedule, PairType, PairsResponse,
        PoolInfoResponse, RewardInfo, ScheduleResponse,
    },
};

#[cw_serde]
pub enum ExecuteMsg {
    /// Setup generators with their respective allocation points.
    /// Only the owner or generator controller can execute this.
    SetupPools {
        /// The list of pools with allocation point.
        pools: Vec<(String, Uint128)>,
    },
    /// Update rewards and return it to user.
    ClaimRewards {
        /// The LP token cw20 address or token factory denom
        lp_tokens: Vec<String>,
        user: Option<String>,
    },
    /// Receives a message of type [`Cw20ReceiveMsg`]. Handles cw20 LP token deposits.
    Receive(Cw20ReceiveMsg),
    /// Stake LP tokens in the Generator. LP tokens staked on behalf of recipient if recipient is set.
    /// Otherwise LP tokens are staked on behalf of message sender.
    Deposit { recipient: Option<String> },
    /// Withdraw LP tokens from the Generator
    Withdraw {
        /// The LP token cw20 address or token factory denom
        lp_token: String,
        /// The amount to withdraw. Must not exceed total staked amount.
        amount: Uint128,
        user: Option<String>,
    },
    /// Set a new amount of PADEX to distribute per seconds.
    /// Only the owner can execute this.
    SetTokensPerSecond {
        /// The new amount of PADEX to distribute per second
        amount: Uint128,
    },
    /// Incentivize a pool with external rewards. Rewards can be in either native or cw20 form.
    /// Incentivizor must send incentivization fee along with rewards (if this reward token is new in this pool).
    /// 3rd parties are encouraged to keep endless schedules without breaks even with the small rewards.
    /// Otherwise, reward token will be removed from the pool info and go to outstanding rewards.
    /// Next schedules with the same token will be considered as "new".  
    /// NOTE: Sender must approve allowance for cw20 reward tokens to this contract.
    Incentivize {
        /// The LP token cw20 address or token factory denom
        lp_token: String,
        /// Incentives schedule
        schedule: InputSchedule,
    },
    /// Remove specific reward token from the pool.
    /// Only the owner can execute this.
    RemoveRewardFromPool {
        /// The LP token cw20 address or token factory denom
        lp_token: String,
        /// The reward token cw20 address or token factory denom
        reward: String,
        /// If there is too much spam in the state, owner can bypass upcoming schedules;
        /// Tokens from these schedules will stuck in Generator balance forever.
        /// Set true only in emergency cases i.e. if deregistration message hits gas limit during simulation.
        /// Default: false
        #[serde(default)]
        bypass_upcoming_schedules: bool,
        /// Receiver of unclaimed rewards
        receiver: String,
    },
    /// Claim all or up to the limit accumulated orphaned rewards.
    /// Only the owner can execute this.
    ClaimOrphanedRewards {
        /// Number of assets to claim
        limit: Option<u8>,
        /// Receiver of orphaned rewards
        receiver: String,
    },
    /// Update config.
    /// Only the owner can execute it.
    UpdateConfig {
        /// The new generator controller contract address
        generator_controller: Option<String>,
        /// New incentivization fee info
        incentivization_fee_info: Option<IncentivizationFeeInfo>,
    },
    /// Add or remove token to the block list.
    /// Only owner or guardian can execute this.
    /// Pools which contain these tokens can't be incentivized with PADEX rewards.
    /// Also blocked tokens can't be used as external reward.
    /// Current active pools with these tokens will be removed from active set.
    UpdateBlockedTokenslist {
        /// Tokens to add
        #[serde(default)]
        add: Vec<AssetInfo>,
        /// Tokens to remove
        #[serde(default)]
        remove: Vec<AssetInfo>,
    },
    /// Only factory can set the allocation points to zero for the specified pool.
    /// Initiated from deregistration context in factory.
    DeactivatePool { lp_token: String },
    /// Go through active pools and deactivate the ones which pair type is blocked
    DeactivateBlockedPools {},
    /// Creates a request to change contract ownership
    /// Only the current owner can execute this.
    ProposeNewOwner {
        /// The newly proposed owner
        owner: String,
        /// The validity period of the proposal to change the contract owner
        expires_in: u64,
    },
    /// Removes a request to change contract ownership
    /// Only the current owner can execute this
    DropOwnershipProposal {},
    /// Claims contract ownership
    /// Only the newly proposed owner can execute this
    ClaimOwnership {},
    SetBridge {
        erc20_address: String,
        chain_reference_id: String,
    },
}

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub trader: String,
    pub factory: String,
    pub incentivization_fee_info: Option<IncentivizationFeeInfo>,
    pub padex_name: String,
    pub padex_symbol: String,
    pub padex_description: Option<String>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Config returns the main contract parameters
    #[returns(Config)]
    Config {},
    /// Deposit returns the LP token amount deposited in a specific generator
    #[returns(Uint128)]
    Deposit { lp_token: String, user: String },
    /// PendingToken returns the amount of rewards that can be claimed by an account that deposited a specific LP token in a generator
    #[returns(Vec<Asset>)]
    PendingRewards { lp_token: String, user: String },
    /// RewardInfo returns reward information for a specified LP token
    #[returns(Vec<RewardInfo>)]
    RewardInfo { lp_token: String },
    /// PoolInfo returns information about a pool associated with the specified LP token
    #[returns(PoolInfoResponse)]
    PoolInfo { lp_token: String },
    /// Returns a list of tuples with addresses and their staked amount
    #[returns(Vec<(String, Uint128)>)]
    PoolStakers {
        lp_token: String,
        start_after: Option<String>,
        limit: Option<u8>,
    },
    /// Returns paginated list of blocked tokens
    #[returns(Vec<AssetInfo>)]
    BlockedTokensList {
        start_after: Option<AssetInfo>,
        limit: Option<u8>,
    },
    /// Checks whether fee expected for the specified pool if user wants to add new reward schedule
    #[returns(bool)]
    IsFeeExpected { lp_token: String, reward: String },
    /// Returns the list of all external reward schedules for the specified LP token
    #[returns(Vec<ScheduleResponse>)]
    ExternalRewardSchedules {
        /// Reward cw20 addr/denom
        reward: String,
        lp_token: String,
        /// Start after specified timestamp
        start_after: Option<u64>,
        /// Limit number of returned schedules.
        limit: Option<u8>,
    },
    #[returns(Vec<String>)]
    /// Returns the list of all ever incentivized pools
    ListPools {
        /// Start after specified LP token
        start_after: Option<String>,
        /// Limit number of returned pools.
        limit: Option<u8>,
    },
    #[returns(Vec<(String, Uint128)>)]
    /// Returns the list of all pools receiving padex emissions
    ActivePools {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum FactoryQueryMsg {
    #[returns(Vec<PairType>)]
    BlacklistedPairTypes {},
    #[returns(FeeInfoResponse)]
    FeeInfo {
        /// The pair type for which we return fee information. Pair type is a [`PairType`] struct
        pair_type: PairType,
    },
    #[returns(PairInfo)]
    Pair {
        /// The assets for which we return a pair
        asset_infos: Vec<AssetInfo>,
    },
    #[returns(PairsResponse)]
    Pairs {
        /// The pair item to start reading from. It is an [`Option`] type that accepts [`AssetInfo`] elements.
        start_after: Option<Vec<AssetInfo>>,
        /// The number of pairs to read and return. It is an [`Option`] type.
        limit: Option<u32>,
    },
}
