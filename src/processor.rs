use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    // program_error::ProgramError,
    msg,
    pubkey::Pubkey,
    // program_pack::{Pack},
    sysvar::{rent::Rent, Sysvar},
    program::{invoke},
    // clock::{Clock},
    // program_option::COption,
    // system_program::check_id,
    system_instruction
};

use crate::{
    instruction::ClaimTokenInstruction,
    error::DistributorError
};

// use std::convert::TryInto;
use borsh::{BorshSerialize, BorshDeserialize};

pub fn process_instruction<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    input: &[u8],
) -> ProgramResult {
    let instruction = ClaimTokenInstruction::try_from_slice(input)?;
    match instruction {
        ClaimTokenInstruction::CreateTokenDistributor(args) => {
            msg!("Instruction: Create Token Distributor");
            process_create_distributor(
                program_id,
                accounts,
            )
        },
    }
}

pub fn process_create_distributor<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let distribution_authority_account_info = next_account_info(account_info_iter)?;
    let distributor_state_account_info = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

    // check distribution_authority is the tx signer
    if !distribution_authority_account_info.is_signer {
        return Err(DistributorError::IncorrectSigner.into());
    }

    // check program is owner of the distributor_state_account
    if distributor_state_account_info.owner != program_id {
        return Err(DistributorError::IncorrectOwner.into());
    }

    // check distributor_state_account has enough lamports to be rent exempt
    if !rent.is_exempt(distributor_state_account_info.lamports(), distributor_state_account_info.data_len()) {
        return Err(DistributorError::NotRentExempt.into());
    }
   
    Ok(())
}