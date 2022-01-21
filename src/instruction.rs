// use solana_program::program_error::ProgramError;
// use crate::error::CasinoError::InvalidInstruction;
// use std::convert::TryInto;
use borsh::{BorshSerialize, BorshDeserialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    sysvar,
};

use spl_token::ID;

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
/// Args for create game
pub struct CreateTokenDistributorArgs {
    pub reward_amount: u64,
    pub start_ts: i64,
    pub collection_name: String,
}

/// Instructions supported by the Casino program.
#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub enum ClaimTokenInstruction {
    // [signer] authority_account
    // [writable] distributor_state_account
    // [writable] reward_token_account
    // [] collection_creator_account
    // [] rent sysvar
    // [] token_program_account
    CreateTokenDistributor(CreateTokenDistributorArgs),
}

/// Creates an CreateTokenDistributor instruction
#[allow(clippy::too_many_arguments)]
pub fn create_token_distributor(
    program_id: Pubkey,
    authority_account: Pubkey,
    distributor_state_account: Pubkey,
    reward_token_account: Pubkey,
    collection_creator_account: Pubkey,
    reward_amount: u64,
    start_ts: i64,
    collection_name: String,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(authority_account, true),
            AccountMeta::new(distributor_state_account, false),
            AccountMeta::new(reward_token_account, false),
            AccountMeta::new_readonly(collection_creator_account, false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(ID, false),
        ],
        data: ClaimTokenInstruction::CreateTokenDistributor(CreateTokenDistributorArgs {
            reward_amount,
            start_ts,
            collection_name
        })
        .try_to_vec()
        .unwrap(),
    }
}