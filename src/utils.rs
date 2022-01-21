use borsh::{BorshDeserialize};
use solana_program::{
    borsh::try_from_slice_unchecked,
    program_error::ProgramError,
};
use crate::{
    error::DistributorError
};

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