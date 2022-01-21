// use solana_program::program_error::ProgramError;
// use crate::error::CasinoError::InvalidInstruction;
// use std::convert::TryInto;
use borsh::{BorshSerialize, BorshDeserialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    sysvar,
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
/// Args for create game
pub struct CreateTokenDistributorArgs {

}

/// Instructions supported by the Casino program.
#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub enum ClaimTokenInstruction {

    CreateTokenDistributor(CreateTokenDistributorArgs),

}
