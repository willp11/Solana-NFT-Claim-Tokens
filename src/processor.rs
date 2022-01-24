use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
    program_pack::{Pack},
    sysvar::{rent::Rent, Sysvar},
    program::{invoke, invoke_signed},
    clock::{Clock},
    system_program::{check_id}
};

use spl_token::state::Account as TokenAccount;

use spl_token_metadata::state::Metadata as MetadataAccount;
use spl_token_metadata::error::MetadataError;

use crate::{
    instruction::ClaimTokenInstruction,
    error::DistributorError,
    utils::PREFIX,
    utils::create_or_allocate_account_raw,
    state::DistributorAccount,
    state::ProofOfReceiptAccount
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
                args.collection_symbol
            )
        },
        ClaimTokenInstruction::ClaimTokens() => {
            msg!("Instruction: Claim Tokens");
            process_claim_tokens(
                program_id,
                accounts,
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
    collection_symbol: String,
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
    if  reward_token_account.amount < reward_amount_total {
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
    msg!("Calling the token program to transfer ownership authority to PDA...");
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
    distributor_state_account.collection_symbol = collection_symbol;
    distributor_state_account.collection_creator = *collection_creator_account_info.key;

    // pack the distributor_state_account
    distributor_state_account.serialize(&mut &mut distributor_state_account_info.data.borrow_mut()[..])?;
   
    Ok(())
}

pub fn process_claim_tokens<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let claimant_main_account_info = next_account_info(account_info_iter)?;
    let distributor_state_account_info = next_account_info(account_info_iter)?;
    let distributor_reward_account_info = next_account_info(account_info_iter)?;
    let claimant_reward_account_info = next_account_info(account_info_iter)?;
    let pda_account_info = next_account_info(account_info_iter)?;
    let claimant_nft_account_info = next_account_info(account_info_iter)?;
    let nft_metadata_account_info = next_account_info(account_info_iter)?;
    let proof_receipt_account_info = next_account_info(account_info_iter)?;
    let clock = &Clock::from_account_info(next_account_info(account_info_iter)?)?;
    let rent_account = next_account_info(account_info_iter)?;
    let token_program_account = next_account_info(account_info_iter)?;
    spl_token::check_program_account(token_program_account.key)?;
    let system_program_account = next_account_info(account_info_iter)?;
    if check_id(system_program_account.key) == false {
        return Err(DistributorError::InvalidSystemProgram.into());
    }

    // check claimant_main_account_info is the tx signer
    if !claimant_main_account_info.is_signer {
        return Err(DistributorError::IncorrectSigner.into());
    }

    // check program is owner of the distributor_state_account_info
    if distributor_state_account_info.owner != program_id {
        return Err(DistributorError::IncorrectOwner.into());
    }

    // unpack distributor state
    let mut distributor_state_account = DistributorAccount::from_account_info(&distributor_state_account_info)?;

    // check the current ts is after start_ts
    if clock.unix_timestamp < distributor_state_account.start_ts {
        return Err(DistributorError::DistributionNotStarted.into());
    }

    // check the claimant_nft_account_info "owner" == claimant_main_account_info
    let claimant_nft_account = TokenAccount::unpack(&claimant_nft_account_info.data.borrow())?;
    if claimant_nft_account.owner != *claimant_main_account_info.key {
        return Err(DistributorError::IncorrectOwner.into());
    }
 
    // pda derived from "metadata", metadata program id, mint account pubkey
    let metadata_prefix: &str = "metadata";
    let metadata_seeds = &[
        metadata_prefix.as_bytes(),
        spl_token_metadata::ID.as_ref(),
        claimant_nft_account.mint.as_ref()
    ];
    // check the nft_metadata_account_info is derived from the claimant_nft_account_info mint and metadata prefix - ensures we have the correct metadata account
    let (metadata_account_pubkey, _bump_seed) = Pubkey::find_program_address(metadata_seeds, &spl_token_metadata::ID);
    if *nft_metadata_account_info.key != metadata_account_pubkey {
        return Err(DistributorError::InvalidMetadataAccount.into());
    }

    // check the metadata account data - find the creator in metadata creators
    let nft_metadata_account = MetadataAccount::from_account_info(&nft_metadata_account_info)?;
    if let Some(creators) = &nft_metadata_account.data.creators {
        let mut found = false;
        for creator in creators {
            if creator.address == distributor_state_account.collection_creator {
                found = true;
                break;
            }
            creator.address.log();
        }
        if !found {
            return Err(MetadataError::CreatorNotFound.into());
        }
    } else {
        return Err(MetadataError::NoCreatorsPresentOnMetadata.into());
    }

    // collection symbol must be same as in distributor state
    // let symbol = &nft_metadata_account.data.symbol;
    // msg!("symbol: {}", symbol);
    // msg!("symbol in state: {}", distributor_state_account.collection_symbol);
    // if *symbol != distributor_state_account.collection_symbol {
    //     return Err(DistributorError::IncorrectSymbol.into());
    // }

    // check distributor_reward_account_info is same as in distributor state
    if *distributor_reward_account_info.key != distributor_state_account.reward_token_account {
        return Err(DistributorError::InvalidAccounts.into());
    }
 
    // get the PDA account Pubkey (derived from the distributor_state_account_info Pubkey and prefix "distributor")
    let distributor_seeds = &[
        PREFIX.as_bytes(),
        distributor_state_account_info.key.as_ref(),
    ];
    let (reward_account_pda, bump_seed) = Pubkey::find_program_address(distributor_seeds, program_id);

    // transfer tokens to claimant_reward_account from distributor_reward_account_info (pda_account signs)
    let transfer_to_claimant_ix = spl_token::instruction::transfer(
        token_program_account.key, 
        distributor_reward_account_info.key, // src
        claimant_reward_account_info.key, // dst
        &reward_account_pda, // authority
        &[&reward_account_pda], 
        distributor_state_account.reward_amount_per_nft,
    )?;
    msg!("Calling the token program to transfer tokens to claimant account");
    let distributor_transfer_seeds = &[
        PREFIX.as_bytes(),
        distributor_state_account_info.key.as_ref(),
        &[bump_seed]
    ];
    invoke_signed(
        &transfer_to_claimant_ix,
        &[
            distributor_reward_account_info.clone(),
            claimant_reward_account_info.clone(),
            pda_account_info.clone(),
            token_program_account.clone(),
        ],
        &[distributor_transfer_seeds]
    )?;
    // increment the distributor state amount claimed
    distributor_state_account.amount_claimed += distributor_state_account.reward_amount_per_nft;

    // pack the distributor state
    distributor_state_account.serialize(&mut &mut distributor_state_account_info.data.borrow_mut()[..])?;

    // Proof of receipt account
    // get account pubkey of account derived from nft mint and "claimed"
    pub const SEED_STR: &str = "claimed";
    let find_receipt_seed = &[
        SEED_STR.as_bytes(),
        claimant_nft_account.mint.as_ref(),
    ];

    // check the proof of receipt account given is the correct one
    let (proof_of_receipt_pubkey, bump_seed) = Pubkey::find_program_address(find_receipt_seed, program_id);

    if proof_of_receipt_pubkey != *proof_receipt_account_info.key {
        return Err(DistributorError::InvalidAccounts.into());
    }

    let receipt_authority_seeds = &[
        SEED_STR.as_bytes(),
        claimant_nft_account.mint.as_ref(),
        &[bump_seed],
    ];

    // create the account
    create_or_allocate_account_raw(
        *program_id,
        proof_receipt_account_info,
        rent_account,
        system_program_account,
        claimant_main_account_info,
        1,
        receipt_authority_seeds
    )?;

    // unpack the proof of receipt account data
    let mut proof_of_receipt_account = ProofOfReceiptAccount::from_account_info(&proof_receipt_account_info)?;
    // check that tokens have not already been claimed
    if proof_of_receipt_account.received_tokens == true {
        return Err(DistributorError::TokensAlreadyClaimed.into());
    }    

    // set proof of receipt account received_tokens true
    proof_of_receipt_account.received_tokens = true;

    // pack proof of receipt state
    proof_of_receipt_account.serialize(&mut &mut proof_receipt_account_info.data.borrow_mut()[..])?;

    Ok(())
}