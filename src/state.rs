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
pub const MAX_NAME_LENGTH: usize = 32;
pub const MAX_DISTRIBUTOR_DATA_LENGTH: usize = 1 + 32 + 32 + 32 + 32 + 8 + 8 + MAX_NAME_LENGTH + 32;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct DistributorAccount {
    pub is_initialized: bool,
    pub authority: Pubkey, 
    pub reward_token_account: Pubkey,
    pub reward_mint: Pubkey,
    pub reward_amount: u64,
    pub start_ts: i64,
    pub collection_name: String,
    pub collection_creator: Pubkey // candy machine (/the first creator in token metadata)
}

impl DistributorAccount {
    pub fn from_account_info(a: &AccountInfo) -> Result<DistributorAccount, ProgramError> {
        let distributor: DistributorAccount =
            try_from_slice_checked(&a.data.borrow_mut(), MAX_DISTRIBUTOR_DATA_LENGTH)?;

        Ok(distributor)
    }
}