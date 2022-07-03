use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::clock::Slot;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::PUBKEY_BYTES;
use solana_program::{msg, pubkey::Pubkey};

use crate::solana_program::program_pack::{IsInitialized, Pack, Sealed};
use crate::state::{PROGRAM_VERSION, UNINITIALIZED_VERSION};
use solana_maths::Decimal;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StakingPool {
    /// Version of the struct
    pub version: u8,
    pub owner_authority: Pubkey,
    pub admin_authority: Pubkey,
    pub reward_token_pool: Pubkey,
    pub last_update: Slot,
    // last time the state changes
    pub end_time: Slot,
    pub earliest_reward_claim_time: Slot,
    pub duration: u64,
    pub rate_per_slot: Decimal,
    pub cumulative_rate: Decimal,
    pub pool_size: u64,
    pub bump_seed_staking_program: u8,
    pub reserve_fields1: [u8; 32], // since rust on implement traits for array from 0..33 len
    pub reserve_fields2: [u8; 32],
    pub reserve_fields3: [u8; 32],
    pub reserve_fields4: [u8; 32],
}

impl Sealed for StakingPool {}
impl IsInitialized for StakingPool {
    fn is_initialized(&self) -> bool {
        self.version != UNINITIALIZED_VERSION
    }
}
impl Pack for StakingPool {
    const LEN: usize = 1
        + PUBKEY_BYTES
        + PUBKEY_BYTES
        + PUBKEY_BYTES
        + 8
        + 8
        + 8
        + 8
        + Decimal::LEN
        + Decimal::LEN
        + 8
        + 1
        + 128;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let output = array_mut_ref![dst, 0, StakingPool::LEN];
        #[allow(clippy::ptr_offset_with_cast)]
        let (
            version,
            owner_authority,
            admin_authority,
            supply_pubkey,
            last_update,
            end_time,
            duration,
            earliest_reward_claim_time,
            rate_per_slot,
            cumulative_rate,
            pool_size,
            bump_seed_staking_program,
            _,
        ) = mut_array_refs![
            output,
            1,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            8,
            8,
            8,
            8,
            Decimal::LEN,
            Decimal::LEN,
            8,
            1,
            128
        ];
        *version = self.version.to_le_bytes();
        owner_authority.copy_from_slice(self.owner_authority.as_ref());
        admin_authority.copy_from_slice(self.admin_authority.as_ref());
        supply_pubkey.copy_from_slice(self.reward_token_pool.as_ref());
        *last_update = self.last_update.to_le_bytes();
        *end_time = self.end_time.to_le_bytes();
        *duration = self.duration.to_le_bytes();
        *earliest_reward_claim_time = self.earliest_reward_claim_time.to_le_bytes();
        self.rate_per_slot.pack_into_slice(rate_per_slot);
        self.cumulative_rate.pack_into_slice(cumulative_rate);
        *pool_size = self.pool_size.to_le_bytes();
        *bump_seed_staking_program = self.bump_seed_staking_program.to_le_bytes();
    }
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![src, 0, StakingPool::LEN];
        #[allow(clippy::ptr_offset_with_cast)]
        let (
            version,
            owner_authority,
            admin_authority,
            supply_pubkey,
            last_update,
            end_time,
            duration,
            earliest_reward_claim_time,
            rate_per_slot,
            cumulative_rate,
            pool_size,
            bump_seed_staking_program,
            _,
        ) = array_refs![
            input,
            1,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            8,
            8,
            8,
            8,
            Decimal::LEN,
            Decimal::LEN,
            8,
            1,
            128
        ];
        let version = u8::from_le_bytes(*version);
        if version > PROGRAM_VERSION {
            msg!("staking pool version does not match staking program version");
            return Err(ProgramError::InvalidAccountData);
        }
        let owner_authority = Pubkey::new_from_array(*owner_authority);
        let admin_authority = Pubkey::new_from_array(*admin_authority);
        let supply_pubkey = Pubkey::new_from_array(*supply_pubkey);
        let last_update = Slot::from_le_bytes(*last_update);
        let end_time = Slot::from_le_bytes(*end_time);
        let duration = u64::from_le_bytes(*duration);
        let earliest_reward_claim_time = Slot::from_le_bytes(*earliest_reward_claim_time);
        let rate_per_slot = Decimal::unpack_from_slice(rate_per_slot)?;
        let cumulative_rate = Decimal::unpack_from_slice(cumulative_rate)?;
        let pool_size = u64::from_le_bytes(*pool_size);
        let bump_seed_staking_program = u8::from_le_bytes(*bump_seed_staking_program);
        let reserve_field = [0; 32];
        Ok(StakingPool {
            version,
            owner_authority,
            admin_authority,
            reward_token_pool: supply_pubkey,
            last_update,
            end_time,
            duration,
            earliest_reward_claim_time,
            rate_per_slot,
            cumulative_rate,
            pool_size,
            bump_seed_staking_program,
            reserve_fields1: reserve_field,
            reserve_fields2: reserve_field,
            reserve_fields3: reserve_field,
            reserve_fields4: reserve_field,
        })
    }
}
