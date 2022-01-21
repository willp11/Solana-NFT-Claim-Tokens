use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    // program_error::ProgramError,
    msg,
    pubkey::Pubkey,
    program_pack::{Pack},
    sysvar::{rent::Rent, Sysvar},
    program::{invoke},
    // clock::{Clock},
    // program_option::COption,
    // system_program::check_id,
    // system_instruction
};

use spl_token::state::Account as TokenAccount;

use crate::{
    instruction::ClaimTokenInstruction,
    error::DistributorError,
    utils::PREFIX,
    state::DistributorAccount
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
                args.reward_amount_total,
                args.reward_amount_per_nft,
                args.start_ts,
                args.collection_name
            )
        },
    }
}

pub fn process_create_distributor<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    reward_amount_total: u64,
    reward_amount_per_nft: u64,
    start_ts: i64,
    collection_name: String,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let authority_account_info = next_account_info(account_info_iter)?;
    let distributor_state_account_info = next_account_info(account_info_iter)?;
    let reward_token_account_info = next_account_info(account_info_iter)?;
    let collection_creator_account_info = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
    let token_program_account = next_account_info(account_info_iter)?;
    spl_token::check_program_account(token_program_account.key)?;

    // check authority_account_info is the tx signer
    if !authority_account_info.is_signer {
        return Err(DistributorError::IncorrectSigner.into());
    }

    // check program is owner of the distributor_state_account_info
    if distributor_state_account_info.owner != program_id {
        return Err(DistributorError::IncorrectOwner.into());
    }

    // check distributor_state_account_info has enough lamports to be rent exempt
    if !rent.is_exempt(distributor_state_account_info.lamports(), distributor_state_account_info.data_len()) {
        return Err(DistributorError::NotRentExempt.into());
    }

    // check the reward token account has enough tokens
    let reward_token_account = TokenAccount::unpack(&reward_token_account_info.data.borrow())?;
    if  reward_token_account.amount != reward_amount_total {
        return Err(DistributorError::ExpectedAmountMismatch.into());
    }

    // get the PDA account Pubkey (derived from the distributor_state_account_info Pubkey and prefix "distributor")
    let distributor_seeds = &[
        PREFIX.as_bytes(),
        distributor_state_account_info.key.as_ref(),
    ];
    let (pda, _bump_seed) = Pubkey::find_program_address(distributor_seeds, program_id);

    // call token program, set account owner authority of the reward token account to PDA
    let transfer_authority_change_ix = spl_token::instruction::set_authority(
        token_program_account.key,
        reward_token_account_info.key,
        Some(&pda),
        spl_token::instruction::AuthorityType::AccountOwner,
        authority_account_info.key,
        &[&authority_account_info.key],
    )?;
    msg!("Calling the token program to transfer mint authority to PDA...");
    invoke(
        &transfer_authority_change_ix,
        &[
            reward_token_account_info.clone(),
            authority_account_info.clone(),
            token_program_account.clone(),
        ],
    )?;
    
    // unpack the distributor_state_account_info
    let mut distributor_state_account = DistributorAccount::from_account_info(&distributor_state_account_info)?;

    // write the data to state
    distributor_state_account.is_initialized = true;
    distributor_state_account.authority = *authority_account_info.key;
    distributor_state_account.reward_token_account = *reward_token_account_info.key;
    distributor_state_account.reward_mint = reward_token_account.mint;
    distributor_state_account.reward_amount_total = reward_amount_total;
    distributor_state_account.reward_amount_per_nft = reward_amount_per_nft;
    distributor_state_account.amount_claimed = 0;
    distributor_state_account.start_ts = start_ts;
    distributor_state_account.collection_name = collection_name;
    distributor_state_account.collection_creator = *collection_creator_account_info.key;

    // pack the distributor_state_account
    distributor_state_account.serialize(&mut &mut distributor_state_account_info.data.borrow_mut()[..])?;
   
    Ok(())
}