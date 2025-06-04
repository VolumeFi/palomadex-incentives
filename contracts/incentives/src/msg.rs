use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;
use cw20::Cw20ReceiveMsg;

use crate::types::{AssetInfo, IncentivizationFeeInfo, InputSchedule};



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
    },
    /// Set a new amount of PDEX to distribute per seconds.
    /// Only the owner can execute this.
    SetTokensPerSecond {
        /// The new amount of PDEX to distribute per second
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
        /// The new PDEX token info
        pdex_token: Option<AssetInfo>,
        /// The new vesting contract address
        vesting_contract: Option<String>,
        /// The new generator controller contract address
        generator_controller: Option<String>,
        /// The new generator guardian
        guardian: Option<String>,
        /// New incentivization fee info
        incentivization_fee_info: Option<IncentivizationFeeInfo>,
    },
    /// Add or remove token to the block list.
    /// Only owner or guardian can execute this.
    /// Pools which contain these tokens can't be incentivized with PDEX rewards.
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
}