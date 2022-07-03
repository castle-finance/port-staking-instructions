use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::pubkey::PUBKEY_BYTES;

use crate::solana_program::program_error::ProgramError;
use crate::solana_program::program_pack::{IsInitialized, Pack, Sealed};
use crate::solana_program::{msg, pubkey::Pubkey};
use crate::state::{PROGRAM_VERSION, UNINITIALIZED_VERSION};
use solana_maths::Decimal;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StakeAccount {
    /// Version of the struct
    pub version: u8,
    /// rate when last time the state changes
    pub start_rate: Decimal,
    pub owner: Pubkey,
    pub pool_pubkey: Pubkey,
    pub deposited_amount: u64,
    pub unclaimed_reward_wads: Decimal,
    pub reserve_fields1: [u8; 32],
    // since rust on implement traits for array from 0..33 len
    pub reserve_fields2: [u8; 32],
    pub reserve_fields3: [u8; 32],
    pub reserve_fields4: [u8; 32],
}

impl Sealed for StakeAccount {}
impl IsInitialized for StakeAccount {
    fn is_initialized(&self) -> bool {
        self.version != UNINITIALIZED_VERSION
    }
}
impl Pack for StakeAccount {
    const LEN: usize = 1 + Decimal::LEN + PUBKEY_BYTES + PUBKEY_BYTES + 8 + Decimal::LEN + 128;
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let output = array_mut_ref![dst, 0, StakeAccount::LEN];
        #[allow(clippy::ptr_offset_with_cast)]
        let (version, start_rate, owner, pool_pubkey, deposited_value, unclaimed_reward_wads, _) = mut_array_refs![
            output,
            1,
            Decimal::LEN,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            8,
            Decimal::LEN,
            128
        ];
        *version = self.version.to_le_bytes();
        self.start_rate.pack_into_slice(start_rate);
        owner.copy_from_slice(self.owner.as_ref());
        pool_pubkey.copy_from_slice(self.pool_pubkey.as_ref());
        *deposited_value = self.deposited_amount.to_le_bytes();
        self.unclaimed_reward_wads
            .pack_into_slice(unclaimed_reward_wads);
    }
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![src, 0, StakeAccount::LEN];
        #[allow(clippy::ptr_offset_with_cast)]
        let (version, start_rate, owner, pool_pubkey, deposited_value, unclaimed_reward_wads, _) = array_refs![
            input,
            1,
            Decimal::LEN,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            8,
            Decimal::LEN,
            128
        ];
        let version = u8::from_le_bytes(*version);
        if version > PROGRAM_VERSION {
            msg!("stake account version does not match staking program version");
            return Err(ProgramError::InvalidAccountData);
        }
        let start_rate = Decimal::unpack_from_slice(start_rate)?;
        let owner = Pubkey::new_from_array(*owner);
        let pool_pubkey = Pubkey::new_from_array(*pool_pubkey);
        let deposited_value = u64::from_le_bytes(*deposited_value);
        let unclaimed_reward_wads = Decimal::unpack_from_slice(unclaimed_reward_wads)?;
        let reserve_field = [0; 32];
        Ok(Self {
            version,
            start_rate,
            owner,
            pool_pubkey,
            deposited_amount: deposited_value,
            unclaimed_reward_wads,
            reserve_fields1: reserve_field,
            reserve_fields2: reserve_field,
            reserve_fields3: reserve_field,
            reserve_fields4: reserve_field,
        })
    }
}
