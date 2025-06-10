#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint128};

use crate::asset::{addr_opt_validate, validate_native_denom, AssetInfo};
use crate::error::ContractError;
use crate::msg::InstantiateMsg;
use crate::state::{ACTIVE_POOLS, CONFIG};
use crate::types::{Config, CreateDenomMsg, DenomUnit, Metadata, PalomaMsg};

/// Contract name that is used for migration.
pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
/// Contract version that is used for migration.
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<PalomaMsg>, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    if let Some(fee_info) = &msg.incentivization_fee_info {
        deps.api.addr_validate(fee_info.fee_receiver.as_str())?;
        validate_native_denom(&fee_info.fee.denom)?;
    }

    let subdenom = "pdex";
    let denom_creator = env.contract.address.to_string();
    let denom = "factory/".to_string() + denom_creator.as_str() + "/" + subdenom;
    let pdex_token: AssetInfo = AssetInfo::NativeToken {
        denom: denom.clone(),
    };

    CONFIG.save(
        deps.storage,
        &Config {
            owner: deps.api.addr_validate(&msg.owner)?,
            factory: deps.api.addr_validate(&msg.factory)?,
            generator_controller: None,
            pdex_token: pdex_token.clone(),
            pdex_per_second: Uint128::zero(),
            total_alloc_points: Uint128::zero(),
            guardian: addr_opt_validate(deps.api, &msg.guardian)?,
            incentivization_fee_info: msg.incentivization_fee_info,
        },
    )?;
    ACTIVE_POOLS.save(deps.storage, &vec![])?;

    let metadata: Metadata = Metadata {
        description: msg.pdex_description.unwrap_or_default(),
        denom_units: vec![
            DenomUnit {
                denom: denom.clone(),
                exponent: 0,
                aliases: vec![],
            },
            DenomUnit {
                denom: msg.pdex_symbol.clone(),
                exponent: 6,
                aliases: vec![],
            },
        ],
        name: msg.pdex_name.clone(),
        symbol: msg.pdex_symbol.clone(),
        base: denom.clone(),
        display: msg.pdex_symbol,
    };

    let messages = vec![CosmosMsg::Custom(PalomaMsg::TokenFactoryMsg {
        create_denom: Some(CreateDenomMsg {
            subdenom: subdenom.to_string(),
            metadata,
        }),
        mint_tokens: None,
    })];
    Ok(Response::new().add_messages(messages))
}
