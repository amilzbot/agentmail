use crate::ID as PINOCCHIO_COUNTER_PROGRAM_ID;
use pinocchio::{account::AccountView, error::ProgramError};

/// Verify the account is a system program, returning an error if it is not.
///
/// # Arguments
/// * `account` - The account to verify.
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
#[inline(always)]
pub fn verify_system_program(account: &AccountView) -> Result<(), ProgramError> {
    if account.address() != &pinocchio_system::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}

/// Verify the account is the current program, returning an error if it is not.
///
/// # Arguments
/// * `account` - The account to verify.
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
#[inline(always)]
pub fn verify_current_program(account: &AccountView) -> Result<(), ProgramError> {
    if account.address() != &PINOCCHIO_COUNTER_PROGRAM_ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}

/// Get the current blockchain timestamp.
/// 
/// For now, this is a placeholder that returns a fixed timestamp.
/// In a full implementation, this would access the Clock sysvar.
///
/// # Returns
/// * `Result<i64, ProgramError>` - Unix timestamp or error
#[inline(always)]
pub fn get_current_timestamp() -> Result<i64, ProgramError> {
    // TODO: Access Clock sysvar for real timestamp
    // For hackathon demo, using a fixed recent timestamp
    Ok(1707523200) // 2026-02-10 00:00:00 UTC
}
