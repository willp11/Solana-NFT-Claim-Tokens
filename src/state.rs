use solana_program::{
    pubkey::Pubkey,
    account_info::AccountInfo,
    program_error::ProgramError
};
use borsh::{BorshSerialize, BorshDeserialize};
use crate::{
    utils::try_from_slice_checked
};

// DISTRIBUTOR ACCOUNT
pub const MAX_DISTRIBUTOR_DATA_LENGTH: usize = 1 + 32 + 32 + 32 + 32 + 8 + 8;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct DistributorAccount {
    pub is_initialized: bool,
    pub authority: Pubkey, 
    pub nft_mint: Pubkey,
    pub reward_mint: Pubkey,
    pub reward_pool: Pubkey,
    pub reward_amount: u64,
    pub start_ts: i64
}

impl DistributorAccount {
    pub fn from_account_info(a: &AccountInfo) -> Result<DistributorAccount, ProgramError> {
        let distributor: DistributorAccount =
            try_from_slice_checked(&a.data.borrow_mut(), MAX_DISTRIBUTOR_DATA_LENGTH)?;

        Ok(distributor)
    }
}