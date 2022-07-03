use solana_program::clock::Slot;
use std::convert::TryInto;
use std::mem::size_of;

use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::pubkey::PUBKEY_BYTES;

use crate::error::StakingError;
use crate::instruction::StakingInstruction::*;
use crate::solana_program::{msg, program_error::ProgramError, pubkey::Pubkey, sysvar};

/// Instructions supported by the lending program.
#[derive(Clone, Debug, PartialEq)]
pub enum StakingInstruction {
    /// Accounts expected by this instruction:
    ///   0. `[signer]` Transfer reward token authority.
    ///   1. `[writable]` Reward token supply.
    ///   2. `[writable]` Reward token pool - uninitialized.
    ///   3. `[writable]` Staking pool - uninitialized.
    ///   4. `[]` Reward token mint.
    ///   5. `[]` Staking program derived that owns reward token pool.
    ///   6. `[]` Rent sysvar .
    ///   7. `[]` Token program.
    InitStakingPool {
        supply: u64,   // rate per slot = supply / duration
        duration: u64, // num of slots
        earliest_reward_claim_time: Slot,
        bump_seed_staking_program: u8,
        pool_owner_authority: Pubkey,
        admin_authority: Pubkey,
    },
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` Stake account - uninitialized.
    ///   1. `[]` Staking Pool.
    ///   2. `[]` Stake account owner.
    ///   3. `[]` Rent sysvar.
    CreateStakeAccount,
    /// Deposit to a stake account.
    ///
    /// Accounts expected by this instruction:
    ///   0. `[signer]` authority.
    ///   1. `[writable]` Stake account.
    ///   2. `[writable]` Staking pool.
    ///   3. `[]` Clock sysvar.
    Deposit(u64),

    /// Withdrawn to a stake account.
    ///
    /// Accounts expected by this instruction:
    ///   0. `[signer]` authority.
    ///   1. `[writable]` Stake account.
    ///   2. `[writable]` Staking pool.
    ///   3. `[]` Clock sysvar.
    Withdraw(u64),
    /// Claim all unclaimed Reward from a stake account
    ///
    /// Accounts expected by this instruction:
    ///   0. `[signer]` Stake account owner.
    ///   1. `[writable]` Stake account.
    ///   2. `[writable]` Staking pool.
    ///   3. `[writable]` Reward token pool.
    ///   4. `[writable]` Reward destination.
    ///   5. `[]` Staking Pool owner derived from staking pool pubkey
    ///   6. `[]` Clock sysvar.
    ///   7. `[]` Token program.
    ClaimReward,
}

impl StakingInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        input
            .split_first()
            .ok_or_else(|| StakingError::InstructionUnpackError.into())
            .and_then(|(&tag, rest)| match tag {
                0 => {
                    let (supply, rest) = Self::unpack_u64(rest)?;
                    let (duration, rest) = Self::unpack_u64(rest)?;
                    let (earliest_reward_claim_time, rest) = Self::unpack_u64(rest)?;
                    let (bump_seed_staking_program, rest) = Self::unpack_u8(rest)?;
                    let (pool_owner_authority, rest) = Self::unpack_pubkey(rest)?;
                    let (admin_authority, rest) = Self::unpack_pubkey(rest)?;
                    Ok((
                        InitStakingPool {
                            supply,
                            duration,
                            earliest_reward_claim_time,
                            bump_seed_staking_program,
                            pool_owner_authority,
                            admin_authority,
                        },
                        rest,
                    ))
                }
                1 => Ok((CreateStakeAccount, rest)),
                2 => {
                    let (amount, rest) = Self::unpack_u64(rest)?;
                    Ok((Deposit(amount), rest))
                }
                3 => {
                    let (amount, rest) = Self::unpack_u64(rest)?;
                    Ok((Withdraw(amount), rest))
                }
                4 => Ok((ClaimReward, rest)),
                _ => {
                    msg!("Instruction cannot be unpacked");
                    Err(StakingError::InstructionUnpackError.into())
                }
            })
            .and_then(|(ins, rest)| {
                if rest.is_empty() {
                    Ok(ins)
                } else {
                    Err(StakingError::InstructionUnpackError.into())
                }
            })
    }
    fn unpack_u64(input: &[u8]) -> Result<(u64, &[u8]), ProgramError> {
        if input.len() < 8 {
            msg!("u64 cannot be unpacked");
            return Err(StakingError::InstructionUnpackError.into());
        }
        let (bytes, rest) = input.split_at(8);
        let value = bytes
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(StakingError::InstructionUnpackError)?;
        Ok((value, rest))
    }
    fn unpack_pubkey(input: &[u8]) -> Result<(Pubkey, &[u8]), ProgramError> {
        if input.len() < PUBKEY_BYTES {
            msg!("Pubkey cannot be unpacked");
            return Err(StakingError::InstructionUnpackError.into());
        }
        let (key, rest) = input.split_at(PUBKEY_BYTES);
        let pk = Pubkey::new(key);
        Ok((pk, rest))
    }
    fn unpack_u8(input: &[u8]) -> Result<(u8, &[u8]), ProgramError> {
        if input.is_empty() {
            msg!("u8 cannot be unpacked");
            return Err(StakingError::InstructionUnpackError.into());
        }
        let (bytes, rest) = input.split_at(1);
        let value = bytes
            .get(..1)
            .and_then(|slice| slice.try_into().ok())
            .map(u8::from_le_bytes)
            .ok_or(StakingError::InstructionUnpackError)?;
        Ok((value, rest))
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match *self {
            Self::InitStakingPool {
                supply,
                duration,
                earliest_reward_claim_time,
                bump_seed_staking_program,
                pool_owner_authority,
                admin_authority,
            } => {
                buf.push(0);
                buf.extend_from_slice(&supply.to_le_bytes());
                buf.extend_from_slice(&duration.to_le_bytes());
                buf.extend_from_slice(&earliest_reward_claim_time.to_le_bytes());
                buf.extend_from_slice(&bump_seed_staking_program.to_le_bytes());
                buf.extend_from_slice(pool_owner_authority.as_ref());
                buf.extend_from_slice(admin_authority.as_ref());
            }
            Self::CreateStakeAccount => {
                buf.push(1);
            }
            Self::Deposit(amount) => {
                buf.push(2);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
            Self::Withdraw(amount) => {
                buf.push(3);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
            Self::ClaimReward => {
                buf.push(4);
            }
        };
        buf
    }
}

//helpers
fn create_write_accounts(accounts: Vec<Pubkey>) -> impl Iterator<Item = AccountMeta> {
    accounts.into_iter().map(|acc| AccountMeta::new(acc, false))
}

fn create_read_accounts(accounts: Vec<Pubkey>) -> impl Iterator<Item = AccountMeta> {
    accounts
        .into_iter()
        .map(|acc| AccountMeta::new_readonly(acc, false))
}

pub fn create_stake_account(
    program_id: Pubkey,
    stake_account: Pubkey,
    staking_pool: Pubkey,
    stake_account_owner: Pubkey,
) -> Instruction {
    let read_accounts =
        create_read_accounts(vec![staking_pool, stake_account_owner, sysvar::rent::id()]);

    let accounts = vec![AccountMeta::new(stake_account, false)]
        .into_iter()
        .chain(read_accounts)
        .collect();

    Instruction {
        program_id,
        accounts,
        data: StakingInstruction::CreateStakeAccount.pack(),
    }
}

pub fn claim_reward(
    program_id: Pubkey,
    stake_account_owner: Pubkey,
    stake_account: Pubkey,
    staking_pool: Pubkey,
    reward_token_pool: Pubkey,
    reward_destination: Pubkey,
    sub_reward_pool: Pubkey,
    sub_reward_dest: Pubkey,
) -> Instruction {
    let (staking_program_derived, _bump_seed) =
        Pubkey::find_program_address(&[staking_pool.as_ref()], &program_id);

    let write_accounts = create_write_accounts(vec![
        stake_account,
        staking_pool,
        reward_token_pool,
        reward_destination,
    ]);

    let read_accounts = create_read_accounts(vec![
        staking_program_derived,
        sysvar::clock::id(),
        spl_token::id(),
    ]);

    let sub_reward_accounts = create_write_accounts(vec![sub_reward_pool, sub_reward_dest]);

    let accounts = vec![AccountMeta::new_readonly(stake_account_owner, true)]
        .into_iter()
        .chain(write_accounts)
        .chain(read_accounts)
        .chain(sub_reward_accounts)
        .collect();

    Instruction {
        program_id,
        accounts,
        data: ClaimReward.pack(),
    }
}

/// Creates an InitStakingPool instruction
#[allow(clippy::too_many_arguments)]
pub fn init_staking_pool(
    program_id: Pubkey,
    supply: u64,
    duration: u64,
    earliest_reward_claim_time: Slot,
    transfer_reward_token_authority: Pubkey,
    reward_token_supply: Pubkey,
    reward_token_pool: Pubkey,
    staking_pool: Pubkey,
    reward_token_mint: Pubkey,
    staking_pool_owner_derived: Pubkey,
    admin_authority: Pubkey,
) -> Instruction {
    let (staking_program_derived, bump_seed) =
        Pubkey::find_program_address(&[staking_pool.as_ref()], &program_id);

    let write_accounts =
        create_write_accounts(vec![reward_token_supply, reward_token_pool, staking_pool]);

    let read_accounts = create_read_accounts(vec![
        reward_token_mint,
        staking_program_derived,
        sysvar::rent::id(),
        spl_token::id(),
    ]);

    let accounts = vec![AccountMeta::new_readonly(
        transfer_reward_token_authority,
        true,
    )]
    .into_iter()
    .chain(write_accounts)
    .chain(read_accounts)
    .collect();

    Instruction {
        program_id,
        accounts,
        data: StakingInstruction::InitStakingPool {
            supply,
            duration,
            earliest_reward_claim_time,
            bump_seed_staking_program: bump_seed,
            pool_owner_authority: staking_pool_owner_derived,
            admin_authority,
        }
        .pack(),
    }
}

pub fn deposit(
    program_id: Pubkey,
    amount: u64,
    authority: Pubkey,
    stake_account: Pubkey,
    staking_pool: Pubkey,
) -> Instruction {
    let write_accounts = create_write_accounts(vec![stake_account, staking_pool]);
    let accounts = vec![AccountMeta::new_readonly(authority, true)]
        .into_iter()
        .chain(write_accounts)
        .chain(vec![AccountMeta::new_readonly(sysvar::clock::id(), false)])
        .collect();

    Instruction {
        program_id,
        accounts,
        data: Deposit(amount).pack(),
    }
}

pub fn withdraw(
    program_id: Pubkey,
    amount: u64,
    authority: Pubkey,
    stake_account: Pubkey,
    staking_pool: Pubkey,
) -> Instruction {
    let write_accounts = create_write_accounts(vec![stake_account, staking_pool]);

    let accounts = vec![AccountMeta::new_readonly(authority, true)]
        .into_iter()
        .chain(write_accounts)
        .chain(vec![AccountMeta::new_readonly(sysvar::clock::id(), false)])
        .collect();

    Instruction {
        program_id,
        accounts,
        data: Withdraw(amount).pack(),
    }
}
