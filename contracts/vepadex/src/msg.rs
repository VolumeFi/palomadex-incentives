use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, CustomMsg, Uint128};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cw_serde]
pub struct InstantiateMsg {
    pub lock_denom: String,
    pub owner: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    // Reserve for LP staking in the future
    Receive(Cw20ReceiveMsg),
    CreateLock {
        // unlock_week specifies the week at which to unlock
        // in units of weeks since the epoch
        end_lock_time: u64,
        user: Option<String>,
    },
    IncreaseLockAmount {
        user: Option<String>,
    },
    Withdraw {
        user: Option<String>,
    },
    Checkpoint {},
    IncreaseEndLockTime {
        // unlock_week specifies the week at which to unlock
        // in units of weeks since the epoch
        end_lock_time: u64,
        user: Option<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(StateResponse)]
    State { timestamp: Option<u64> },
    #[returns(LockerResponse)]
    Locker {
        address: String,
        timestamp: Option<u64>,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub lock_denom: String,
}

#[derive(Default, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub total_deposited_amount: Uint128,
    pub total_locked_amount: Uint128,
    pub total_balance: Uint128,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct LockerResponse {
    pub deposited_amount: Uint128,
    pub locked_amount: Uint128,
    pub balance: Uint128,
}

#[cw_serde]
pub enum PalomaMsg {
    TokenFactoryMsg {
        create_denom: Option<CreateDenomMsg>,
        mint_tokens: Option<MintMsg>,
    },
}

#[cw_serde]
pub struct CreateDenomMsg {
    pub subdenom: String,
    pub metadata: Metadata,
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
pub struct DenomUnit {
    pub denom: String,
    pub exponent: u32,
    pub aliases: Vec<String>,
}

#[cw_serde]
pub struct MintMsg {
    pub denom: String,
    pub amount: Uint128,
    pub mint_to_address: String,
}

#[cw_serde]
pub struct StakingResponse {
    /// The ASTRO denom
    pub deposit_amount: Uint128,
    /// The xASTRO denom
    pub padex_amount: Uint128,
}

impl CustomMsg for PalomaMsg {}
