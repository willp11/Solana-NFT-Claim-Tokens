use borsh::{BorshDeserialize};
use solana_program::{
    borsh::try_from_slice_unchecked,
    program_error::ProgramError,
    system_instruction,
    account_info::AccountInfo,
    pubkey::Pubkey,
    entrypoint::ProgramResult,
    sysvar::{rent::Rent, Sysvar},
    msg,
    program::{invoke, invoke_signed},
};
use std::convert::TryInto;
use crate::{
    error::DistributorError
};

pub const PREFIX: &str = "distributor";

pub fn try_from_slice_checked<T: BorshDeserialize>(
    data: &[u8],
    data_size: usize,
) -> Result<T, ProgramError> {
    if data.len() != data_size
    {
        return Err(DistributorError::DataTypeMismatch.into());
    }

    let result: T = try_from_slice_unchecked(data)?;

    Ok(result)
}

/// Create account almost from scratch, lifted from
/// https://github.com/solana-labs/solana-program-library/tree/master/associated-token-account/program/src/processor.rs#L51-L98
#[inline(always)]
pub fn create_or_allocate_account_raw<'a>(
    program_id: Pubkey,
    new_account_info: &AccountInfo<'a>,
    rent_sysvar_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    size: usize,
    signer_seeds: &[&[u8]],
) -> ProgramResult {
    let rent = &Rent::from_account_info(rent_sysvar_info)?;
    let required_lamports = rent
        .minimum_balance(size)
        .max(1)
        .saturating_sub(new_account_info.lamports());

    if required_lamports > 0 {
        msg!("Transfer {} lamports to the new account", required_lamports);
        invoke(
            &system_instruction::transfer(&payer_info.key, new_account_info.key, required_lamports),
            &[
                payer_info.clone(),
                new_account_info.clone(),
                system_program_info.clone(),
            ],
        )?;
    }

    let accounts = &[new_account_info.clone(), system_program_info.clone()];

    msg!("Allocate space for the account");
    invoke_signed(
        &system_instruction::allocate(new_account_info.key, size.try_into().unwrap()),
        accounts,
        &[&signer_seeds],
    )?;

    msg!("Assign the account to the owning program");
    invoke_signed(
        &system_instruction::assign(new_account_info.key, &program_id),
        accounts,
        &[&signer_seeds],
    )?;

    Ok(())
}