pub use stake_account::*;
pub use staking_pool::*;

pub mod stake_account;
pub mod staking_pool;

/// Current version of the program and all new accounts created
pub const PROGRAM_VERSION: u8 = 1;
/// Accounts are created with data zeroed out, so uninitialized state instances
/// will have the version set to 0.
pub const UNINITIALIZED_VERSION: u8 = 0;
