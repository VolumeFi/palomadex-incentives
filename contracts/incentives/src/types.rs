use std::fmt::{Display, Formatter, Result};

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin};

use crate::asset::Asset;


#[cw_serde]
pub struct InputSchedule {
    pub reward: Asset,
    pub duration_periods: u64,
}

#[cw_serde]
#[derive(Hash, Eq)]
pub enum AssetInfo {
    /// Non-native Token
    Token { contract_addr: Addr },
    /// Native token
    NativeToken { denom: String },
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
}
