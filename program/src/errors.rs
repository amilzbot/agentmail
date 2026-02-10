use codama::CodamaErrors;
use pinocchio::error::ProgramError;
use thiserror::Error;

/// Errors that may be returned by the Pinocchio Counter Program.
#[derive(Clone, Debug, Eq, PartialEq, Error, CodamaErrors)]
pub enum PinocchioCounterProgramError {
    /// (0) Authority invalid or does not match counter authority
    #[error("Authority invalid or does not match counter authority")]
    InvalidAuthority,

    /// (1) Event authority PDA is invalid
    #[error("Event authority PDA is invalid")]
    InvalidEventAuthority,
}

impl From<PinocchioCounterProgramError> for ProgramError {
    fn from(e: PinocchioCounterProgramError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

/// Errors that may be returned by the AgentMail Program.
#[derive(Clone, Debug, Eq, PartialEq, Error, CodamaErrors)]
pub enum AgentMailProgramError {
    /// (0) Authority invalid or does not match registry authority
    #[error("Authority invalid or does not match registry authority")]
    InvalidAuthority,

    /// (1) Agent name is too long (max 64 bytes)
    #[error("Agent name is too long (max 64 bytes)")]
    NameTooLong,

    /// (2) Inbox URL is too long (max 256 bytes)
    #[error("Inbox URL is too long (max 256 bytes)")]
    InboxUrlTooLong,

    /// (3) Invalid name length in stored data
    #[error("Invalid name length in stored data")]
    InvalidNameLength,

    /// (4) Invalid inbox URL length in stored data  
    #[error("Invalid inbox URL length in stored data")]
    InvalidInboxUrlLength,

    /// (5) Invalid UTF-8 data in string fields
    #[error("Invalid UTF-8 data in string fields")]
    InvalidUtf8,

    /// (6) Agent registry already exists for this authority
    #[error("Agent registry already exists for this authority")]
    RegistryAlreadyExists,

    /// (7) Agent registry does not exist for this authority
    #[error("Agent registry does not exist for this authority")]
    RegistryDoesNotExist,

    /// (8) Invalid account size for agent registry
    #[error("Invalid account size for agent registry")]
    InvalidAccountSize,

    /// (9) Invalid account data format
    #[error("Invalid account data format")]
    InvalidAccountData,
}

impl From<AgentMailProgramError> for ProgramError {
    fn from(e: AgentMailProgramError) -> Self {
        ProgramError::Custom(100 + e as u32) // Offset to avoid conflicts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        let error: ProgramError = PinocchioCounterProgramError::InvalidAuthority.into();
        assert_eq!(error, ProgramError::Custom(0));

        let error: ProgramError = PinocchioCounterProgramError::InvalidEventAuthority.into();
        assert_eq!(error, ProgramError::Custom(1));
    }

    #[test]
    fn test_agentmail_error_conversion() {
        let error: ProgramError = AgentMailProgramError::InvalidAuthority.into();
        assert_eq!(error, ProgramError::Custom(100));

        let error: ProgramError = AgentMailProgramError::NameTooLong.into();
        assert_eq!(error, ProgramError::Custom(101));

        let error: ProgramError = AgentMailProgramError::RegistryAlreadyExists.into();
        assert_eq!(error, ProgramError::Custom(106));
    }
}
