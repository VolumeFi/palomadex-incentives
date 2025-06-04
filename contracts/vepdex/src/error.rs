use crate::state::MINIMUM_STAKE_AMOUNT;
use cosmwasm_std::{OverflowError, StdError};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Data should be given")]
    DataShouldBeGiven {},

    #[error("Insufficient funds sent")]
    InsufficientFunds {},

    #[error("Contracts can't create, modify, or withdraw locks")]
    ContractsCannotInteractWithLocks {},

    #[error("A lock already exists. You cannot create a new one until the old one expires and is withdrawn.")]
    LockAlreadyExists {},

    #[error("A lock does not exist. You cannot modify a lock before creating one.")]
    LockDoesNotExist {},

    #[error("The current lock is expired. You cannot modify an expired lock. Please withdraw it and try again.")]
    LockIsExpired {},

    #[error("Insufficient funds sent. Locks must contain a non zero amount.")]
    InsufficientLockAmount {},

    #[error("Insufficient funds sent. Lock amount increases must be greater than 0.")]
    InsufficientLockIncreaseAmount {},

    #[error("Must specify an end lock time at least a week in the future. If a lock already exists, you must specify an end lock time greater than the existing one.")]
    EndLockTimeTooEarly {},

    #[error("Must specify an end lock time less than {max_weeks} into the future. You tried to specify an end lock time {lock_duration_in_weeks} weeks into the future.")]
    EndLockTimeTooLate {
        max_weeks: u64,
        lock_duration_in_weeks: u64,
    },

    #[error("Config contracts have not been registered yet")]
    ConfigContractsNotRegistered {},

    #[error("Initial stake amount must be more than {MINIMUM_STAKE_AMOUNT}")]
    MinimumStakeAmountError {},

    #[error("Insufficient amount of Stake")]
    StakeAmountTooSmall {},
}
