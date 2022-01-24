use thiserror::Error;
use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum DistributorError {
    // Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,

    // Unauthorized account
    #[error("Incorrect signer")]
    IncorrectSigner,

    // Not rent exempt
    #[error("State account not rent exempt")]
    NotRentExempt,

    // Invalid mint
    #[error("Invalid mint")]
    InvalidMint,

    // Expected amount mismatch - wrong number of tokens in temporary token account
    #[error("Expected amount mismatch")]
    ExpectedAmountMismatch,

    // Unauthorized account
    #[error("Unauthorized account")]
    UnauthorizedAccount,

    // Incorrect account owner
    #[error("Incorrect account owner")]
    IncorrectOwner,

    // Invalid accounts
    #[error("Invalid accounts")]
    InvalidAccounts,

    // Invalid metadata account
    #[error("Invalid metadata account")]
    InvalidMetadataAccount,

    // Invalid system program
    #[error("Invalid system program")]
    InvalidSystemProgram,

    // Amount overflow transferring lamports
    #[error("Amount overflow transferring lamports")]
    AmountOverflow,

    // Amount underflow transferring lamports
    #[error("Amount underflow transferring lamports")]
    AmountUnderflow,

    // Data type mismatch
    #[error("Data type mismatch")]
    DataTypeMismatch,

    // Distribution not started
    #[error("Distribution not started")]
    DistributionNotStarted,

    // Incorrect symbol
    #[error("Incorrect symbol in metadata")]
    IncorrectSymbol,

    // Tokens already claimed
    #[error("Tokens already claimed")]
    TokensAlreadyClaimed
}

impl PrintProgramError for DistributorError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl From<DistributorError> for ProgramError {
    fn from(e: DistributorError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for DistributorError {
    fn type_of() -> &'static str {
        "Distributor Error"
    }
}