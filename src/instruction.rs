use borsh::{BorshSerialize, BorshDeserialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    sysvar,
    // _msg
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
/// Args for create game
pub struct CreateTokenDistributorArgs {
    pub reward_amount_total: u64,
    pub reward_amount_per_nft: u64,
    pub start_ts: i64,
    pub collection_symbol: String,
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

    // [signer] claimant_main_account
    // [writable] distributor_state_account (increment amount claimed)
    // [writable] distributor_reward_account (holds the tokens)
    // [writable] claimant_reward_account (receives the tokens)
    // [] pda (has authority to transfer distributor_reward_account tokens)
    // [] claimant_nft_account (holds the claimant's NFT)
    // [] nft_metadata_account (holds the metadata about the NFT account - must match the collection_creator and collection_name fields)
    // [] pda_proof_of_receipt 
    // [] clock sysvar (check now is after start_ts)
    // [] token_program_account (transfers tokens to claimant)
    ClaimTokens(),
}

/// Creates an CreateTokenDistributor instruction
#[allow(clippy::too_many_arguments)]
pub fn create_token_distributor(
    program_id: Pubkey,
    authority_account: Pubkey,
    distributor_state_account: Pubkey,
    reward_token_account: Pubkey,
    collection_creator_account: Pubkey,
    reward_amount_total: u64,
    reward_amount_per_nft: u64,
    start_ts: i64,
    collection_symbol: String,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(authority_account, true),
            AccountMeta::new(distributor_state_account, false),
            AccountMeta::new(reward_token_account, false),
            AccountMeta::new_readonly(collection_creator_account, false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(spl_token::ID, false),
        ],
        data: ClaimTokenInstruction::CreateTokenDistributor(CreateTokenDistributorArgs {
            reward_amount_total,
            reward_amount_per_nft,
            start_ts,
            collection_symbol
        })
        .try_to_vec()
        .unwrap(),
    }
}

/// Creates a ClaimTokens instruction
#[allow(clippy::too_many_arguments)]
pub fn claim_tokens(
    program_id: Pubkey,
    claimant_main_account: Pubkey,
    distributor_state_account: Pubkey,
    distributor_reward_account: Pubkey,
    claimant_reward_account: Pubkey,
    pda_account: Pubkey,
    claimant_nft_account: Pubkey,
    nft_metadata_account: Pubkey,
    proof_of_receipt_account: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(claimant_main_account, true),
            AccountMeta::new(distributor_state_account, false),
            AccountMeta::new(distributor_reward_account, false),
            AccountMeta::new(claimant_reward_account, false),
            AccountMeta::new_readonly(pda_account, false),
            AccountMeta::new_readonly(claimant_nft_account, false),
            AccountMeta::new_readonly(nft_metadata_account, false),
            AccountMeta::new(proof_of_receipt_account, false),
            AccountMeta::new_readonly(sysvar::clock::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(spl_token::ID, false),
        ],
        data: ClaimTokenInstruction::ClaimTokens()
        .try_to_vec()
        .unwrap(),
    }
}