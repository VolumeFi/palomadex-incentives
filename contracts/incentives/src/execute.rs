use std::collections::HashSet;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, ensure, from_json, Addr, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128,
};
use cw_utils::one_coin;
use itertools::Itertools;

use crate::asset::{determine_asset_info, validate_native_denom, Asset, AssetInfo, AssetInfoExt};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, FactoryQueryMsg};
use crate::state::{
    Op, PoolInfo, UserInfo, ACTIVE_POOLS, BLOCKED_TOKENS, CONFIG, OWNERSHIP_PROPOSAL,
};
use crate::types::{Cw20Msg, IncentivizationFeeInfo, PairType, PalomaMsg, SetErc20ToDenom};
use crate::utils::{
    asset_info_key, claim_orphaned_rewards, claim_ownership, claim_rewards,
    deactivate_blocked_pools, deactivate_pool, drop_ownership_proposal, incentivize,
    is_pool_registered, propose_new_owner, query_pair_info, remove_reward_from_pool,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<PalomaMsg>, ContractError> {
    match msg {
        ExecuteMsg::SetupPools { pools } => setup_pools(deps, env, info, pools),
        ExecuteMsg::ClaimRewards { lp_tokens, user } => {
            // Check for duplicated pools
            ensure!(
                lp_tokens.iter().all_unique(),
                ContractError::DuplicatedPoolFound {}
            );

            let user = if user.is_some() {
                assert!(
                    info.sender == CONFIG.load(deps.storage)?.trader,
                    "User address must match sender address"
                );
                user.unwrap()
            } else {
                info.sender.to_string()
            };

            // Collect in-memory mutable objects
            let mut tuples = lp_tokens
                .into_iter()
                .map(|lp_token| {
                    let lp_asset = determine_asset_info(&lp_token, deps.api)?;
                    let pool_info = PoolInfo::load(deps.storage, &lp_asset)?;
                    let user_pos = UserInfo::load_position(deps.storage, &user, &lp_asset)?;
                    Ok((lp_asset, pool_info, user_pos))
                })
                .collect::<Result<Vec<_>, ContractError>>()?;

            // Convert to mutable references
            let mut_tuples = tuples
                .iter_mut()
                .map(|(lp_asset, pool_info, user_pos)| (&*lp_asset, pool_info, user_pos))
                .collect_vec();

            // Compose response. Return early in case of error
            let response = claim_rewards(deps.storage, env, info.sender, &user, mut_tuples)?;

            // Save updates in state
            for (lp_asset, pool_info, user_pos) in tuples {
                pool_info.save(deps.storage, &lp_asset)?;
                user_pos.save(deps.storage, &user, &lp_asset)?;
            }

            Ok(response)
        }
        ExecuteMsg::Receive(cw20msg) => {
            let maybe_lp = Asset::cw20(info.sender, cw20msg.amount);
            let recipient = match from_json(&cw20msg.msg)? {
                Cw20Msg::Deposit { recipient } => recipient,
                Cw20Msg::DepositFor(recipient) => Some(recipient),
            };

            deposit(
                deps,
                env,
                maybe_lp,
                Addr::unchecked(cw20msg.sender),
                recipient,
            )
        }
        ExecuteMsg::Deposit { recipient } => {
            let maybe_lp_coin = one_coin(&info)?;
            let maybe_lp = Asset::native(maybe_lp_coin.denom, maybe_lp_coin.amount);

            deposit(deps, env, maybe_lp, info.sender, recipient)
        }
        ExecuteMsg::Withdraw {
            lp_token,
            amount,
            user,
        } => withdraw(deps, env, info, lp_token, amount, user),
        ExecuteMsg::SetTokensPerSecond { amount } => set_tokens_per_second(deps, env, info, amount),
        ExecuteMsg::Incentivize { lp_token, schedule } => {
            incentivize(deps, info, env, lp_token, schedule)
        }
        ExecuteMsg::RemoveRewardFromPool {
            lp_token,
            reward,
            bypass_upcoming_schedules,
            receiver,
        } => remove_reward_from_pool(
            deps,
            info,
            env,
            lp_token,
            reward,
            bypass_upcoming_schedules,
            receiver,
        ),
        ExecuteMsg::ClaimOrphanedRewards { limit, receiver } => {
            claim_orphaned_rewards(deps, info, limit, receiver)
        }
        ExecuteMsg::UpdateConfig {
            generator_controller,
            incentivization_fee_info,
        } => update_config(deps, info, generator_controller, incentivization_fee_info),
        ExecuteMsg::UpdateBlockedTokenslist { add, remove } => {
            update_blocked_pool_tokens(deps, env, info, add, remove)
        }
        ExecuteMsg::DeactivatePool { lp_token } => deactivate_pool(deps, info, env, lp_token),
        ExecuteMsg::DeactivateBlockedPools {} => deactivate_blocked_pools(deps, env),
        ExecuteMsg::ProposeNewOwner { owner, expires_in } => {
            let config = CONFIG.load(deps.storage)?;

            propose_new_owner(
                deps,
                info,
                env,
                owner,
                expires_in,
                config.owner,
                OWNERSHIP_PROPOSAL,
            )
            .map_err(Into::into)
        }
        ExecuteMsg::DropOwnershipProposal {} => {
            let config = CONFIG.load(deps.storage)?;

            drop_ownership_proposal(deps, info, config.owner, OWNERSHIP_PROPOSAL)
                .map_err(Into::into)
        }
        ExecuteMsg::ClaimOwnership {} => {
            claim_ownership(deps, info, env, OWNERSHIP_PROPOSAL, |deps, new_owner| {
                CONFIG
                    .update::<_, StdError>(deps.storage, |mut v| {
                        v.owner = new_owner;
                        Ok(v)
                    })
                    .map(|_| ())
            })
            .map_err(Into::into)
        }
        ExecuteMsg::SetBridge {
            erc20_address,
            chain_reference_id,
        } => set_bridge(deps, info, erc20_address, chain_reference_id),
    }
}

fn deposit(
    deps: DepsMut,
    env: Env,
    maybe_lp: Asset,
    sender: Addr,
    recipient: Option<String>,
) -> Result<Response<PalomaMsg>, ContractError> {
    let staker = if recipient.is_some() {
        assert!(
            sender == CONFIG.load(deps.storage)?.trader,
            "User address must match sender address"
        );
        recipient.unwrap()
    } else {
        sender.to_string()
    };

    let pair_info = query_pair_info(deps.as_ref(), &maybe_lp.info)?;
    let config = CONFIG.load(deps.storage)?;
    is_pool_registered(
        deps.querier,
        &config,
        &pair_info,
        &maybe_lp.info.to_string(),
    )?;

    let mut pool_info = PoolInfo::may_load(deps.storage, &maybe_lp.info)?.unwrap_or_default();
    let mut user_info = UserInfo::may_load_position(deps.storage, &staker, &maybe_lp.info)?
        .unwrap_or_else(|| UserInfo::new(&env));

    let response = claim_rewards(
        deps.storage,
        env,
        sender,
        &staker,
        vec![(&maybe_lp.info, &mut pool_info, &mut user_info)],
    )?;

    user_info.update_and_sync_position(Op::Add(maybe_lp.amount), &mut pool_info);
    pool_info.save(deps.storage, &maybe_lp.info)?;
    user_info.save(deps.storage, &staker, &maybe_lp.info)?;

    Ok(response.add_attributes([
        attr("action", "deposit"),
        attr("lp_token", maybe_lp.info.to_string()),
        attr("user", staker.as_str()),
        attr("amount", maybe_lp.amount),
    ]))
}

fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    lp_token: String,
    amount: Uint128,
    user: Option<String>,
) -> Result<Response<PalomaMsg>, ContractError> {
    let lp_token_asset = determine_asset_info(&lp_token, deps.api)?;
    let user = if user.is_some() {
        assert!(
            info.sender == CONFIG.load(deps.storage)?.trader,
            "User address must match sender address"
        );
        user.unwrap()
    } else {
        info.sender.to_string()
    };

    let mut user_info = UserInfo::load_position(deps.storage, &user, &lp_token_asset)?;

    if user_info.amount < amount {
        Err(ContractError::AmountExceedsBalance {
            available: user_info.amount,
            withdraw_amount: amount,
        })
    } else {
        let mut pool_info = PoolInfo::load(deps.storage, &lp_token_asset)?;

        let response = claim_rewards(
            deps.storage,
            env,
            info.sender.clone(),
            &user,
            vec![(&lp_token_asset, &mut pool_info, &mut user_info)],
        )?;

        user_info.update_and_sync_position(Op::Sub(amount), &mut pool_info);
        pool_info.save(deps.storage, &lp_token_asset)?;
        if user_info.amount.is_zero() {
            // If user has withdrawn all LP tokens, we can remove his position
            user_info.remove(deps.storage, &user, &lp_token_asset);
        } else {
            user_info.save(deps.storage, &user, &lp_token_asset)?;
        }

        let transfer_msg = lp_token_asset.with_balance(amount).into_msg(info.sender)?;

        Ok(response.add_message(transfer_msg).add_attributes([
            attr("action", "withdraw"),
            attr("lp_token", lp_token_asset.to_string()),
            attr("amount", amount),
        ]))
    }
}

pub fn setup_pools(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    pools: Vec<(String, Uint128)>,
) -> Result<Response<PalomaMsg>, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner && Some(info.sender) != config.generator_controller {
        return Err(ContractError::Unauthorized {});
    }

    let mut pools_set: HashSet<_> = Default::default();
    for (pool, alloc_points) in &pools {
        if alloc_points.is_zero() {
            return Err(ContractError::ZeroAllocPoint {
                lp_token: pool.to_owned(),
            });
        }

        if !pools_set.insert(pool) {
            return Err(ContractError::DuplicatedPoolFound {});
        }
    }

    let blacklisted_pair_types: Vec<PairType> = deps
        .querier
        .query_wasm_smart(&config.factory, &FactoryQueryMsg::BlacklistedPairTypes {})?;

    let setup_pools = pools
        .into_iter()
        .map(|(lp_token, alloc_point)| {
            let maybe_lp = determine_asset_info(&lp_token, deps.api)?;
            let pair_info = query_pair_info(deps.as_ref(), &maybe_lp)?;

            is_pool_registered(deps.querier, &config, &pair_info, &lp_token)?;

            // check if assets in the blocked list
            for asset in &pair_info.asset_infos {
                if BLOCKED_TOKENS.has(deps.storage, &asset_info_key(asset)) {
                    return Err(ContractError::BlockedToken {
                        token: asset.to_string(),
                    });
                }
            }

            // check if pair type is blacklisted
            if blacklisted_pair_types.contains(&pair_info.pair_type) {
                return Err(ContractError::BlockedPairType {
                    pair_type: pair_info.pair_type,
                });
            }

            Ok((maybe_lp, alloc_point))
        })
        .collect::<Result<Vec<_>, ContractError>>()?;

    // Update all reward indexes and remove padex rewards from old active pools
    for (lp_token_asset, _) in ACTIVE_POOLS.load(deps.storage)? {
        let mut pool_info = PoolInfo::load(deps.storage, &lp_token_asset)?;
        pool_info.update_rewards(deps.storage, &env, &lp_token_asset)?;
        pool_info.disable_padex_rewards();
        pool_info.save(deps.storage, &lp_token_asset)?;
    }

    config.total_alloc_points = setup_pools.iter().map(|(_, alloc)| alloc).sum();

    // Set padex rewards for new active pools
    for (active_pool, alloc_points) in &setup_pools {
        let mut pool_info = PoolInfo::may_load(deps.storage, active_pool)?.unwrap_or_default();
        pool_info.update_rewards(deps.storage, &env, active_pool)?;
        pool_info.set_padex_rewards(&config, *alloc_points);
        pool_info.save(deps.storage, active_pool)?;
    }

    ACTIVE_POOLS.save(deps.storage, &setup_pools)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "setup_pools"))
}

fn set_tokens_per_second(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response<PalomaMsg>, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    // Permission check
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let pool_infos = ACTIVE_POOLS
        .load(deps.storage)?
        .into_iter()
        .map(|(lp_token, alloc_points)| {
            let mut pool_info = PoolInfo::load(deps.storage, &lp_token)?;
            pool_info.update_rewards(deps.storage, &env, &lp_token)?;
            Ok((pool_info, lp_token, alloc_points))
        })
        .collect::<StdResult<Vec<_>>>()?;

    config.padex_per_second = amount;

    for (mut pool_info, lp_token, alloc_points) in pool_infos {
        pool_info.set_padex_rewards(&config, alloc_points);
        pool_info.save(deps.storage, &lp_token)?;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "set_tokens_per_second"))
}

fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    generator_controller: Option<String>,
    incentivization_fee_info: Option<IncentivizationFeeInfo>,
) -> Result<Response<PalomaMsg>, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    // Permission check
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let mut attrs = vec![attr("action", "update_config")];

    if let Some(generator_controller) = generator_controller {
        config.generator_controller = Some(deps.api.addr_validate(&generator_controller)?);
        attrs.push(attr("new_generator_controller", generator_controller));
    }

    if let Some(new_info) = incentivization_fee_info {
        deps.api.addr_validate(new_info.fee_receiver.as_str())?;
        validate_native_denom(&new_info.fee.denom)?;
        attrs.push(attr(
            "new_incentivization_fee_receiver",
            &new_info.fee_receiver,
        ));
        attrs.push(attr("new_incentivization_fee", new_info.fee.to_string()));

        config.incentivization_fee_info = Some(new_info);
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attributes(attrs))
}

fn update_blocked_pool_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    add: Vec<AssetInfo>,
    remove: Vec<AssetInfo>,
) -> Result<Response<PalomaMsg>, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    // Permission check
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // Checking for duplicates
    ensure!(
        remove.iter().chain(add.iter()).all_unique(),
        StdError::generic_err("Duplicated tokens found")
    );

    // Remove tokens from blocklist
    for asset_info in remove {
        let asset_info_key = asset_info_key(&asset_info);
        ensure!(
            BLOCKED_TOKENS.has(deps.storage, &asset_info_key),
            StdError::generic_err(format!(
                "Token {asset_info} wasn't found in the blocked list",
            ))
        );

        BLOCKED_TOKENS.remove(deps.storage, &asset_info_key);
    }

    // Add tokens to blocklist
    if !add.is_empty() {
        let active_pools = ACTIVE_POOLS
            .load(deps.storage)?
            .into_iter()
            .map(|(lp_asset, alloc_points)| {
                let asset_infos = query_pair_info(deps.as_ref(), &lp_asset)?.asset_infos;
                Ok((lp_asset, asset_infos, alloc_points))
            })
            .collect::<StdResult<Vec<_>>>()?;

        let mut to_disable = vec![];

        for token_to_block in &add {
            let asset_info_key = asset_info_key(token_to_block);
            if !BLOCKED_TOKENS.has(deps.storage, &asset_info_key) {
                if token_to_block.eq(&config.padex_token) {
                    return Err(StdError::generic_err(format!(
                        "Blocking PADEX token {token_to_block} is prohibited",
                    ))
                    .into());
                }

                for (lp_asset, asset_infos, alloc_points) in &active_pools {
                    if asset_infos.contains(token_to_block) {
                        to_disable.push((lp_asset.clone(), alloc_points));
                    }
                }

                BLOCKED_TOKENS.save(deps.storage, &asset_info_key, &())?;
            } else {
                return Err(StdError::generic_err(format!(
                    "Token {token_to_block} is already in the blocked list",
                ))
                .into());
            }
        }

        if !to_disable.is_empty() {
            let mut reduce_total_alloc_points = Uint128::zero();

            // Update all reward indexes and remove padex rewards from disabled pools
            for (lp_token_asset, alloc_points) in &to_disable {
                let mut pool_info = PoolInfo::load(deps.storage, lp_token_asset)?;
                pool_info.update_rewards(deps.storage, &env, lp_token_asset)?;
                pool_info.disable_padex_rewards();
                pool_info.save(deps.storage, lp_token_asset)?;
                reduce_total_alloc_points += *alloc_points;
            }

            let new_active_pools = active_pools
                .iter()
                .filter_map(|(lp_asset, _, alloc_points)| {
                    if to_disable
                        .iter()
                        .any(|(disable_lp, _)| disable_lp == lp_asset)
                    {
                        None
                    } else {
                        Some((lp_asset.clone(), *alloc_points))
                    }
                })
                .collect_vec();

            config.total_alloc_points = config
                .total_alloc_points
                .checked_sub(reduce_total_alloc_points)?;

            for (lp_asset, alloc_points) in &new_active_pools {
                let mut pool_info = PoolInfo::load(deps.storage, lp_asset)?;
                pool_info.update_rewards(deps.storage, &env, lp_asset)?;
                pool_info.set_padex_rewards(&config, *alloc_points);
                pool_info.save(deps.storage, lp_asset)?;
            }

            ACTIVE_POOLS.save(deps.storage, &new_active_pools)?;
        }
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_tokens_blocklist"))
}

fn set_bridge(
    deps: DepsMut,
    info: MessageInfo,
    erc20_address: String,
    chain_reference_id: String,
) -> Result<Response<PalomaMsg>, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Permission check
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    Ok(Response::new()
        .add_message(CosmosMsg::Custom(PalomaMsg::SkywayMsg {
            set_erc20_to_denom: SetErc20ToDenom {
                erc20_address,
                token_denom: config.padex_token.to_string(),
                chain_reference_id,
            },
        }))
        .add_attribute("action", "set_bridge"))
}
