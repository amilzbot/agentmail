use pinocchio::{account::AccountView, error::ProgramError};

/// Discriminators for the AgentMail Program instructions.
#[repr(u8)]
pub enum AgentMailInstructionDiscriminators {
    RegisterAgent = 3,
    UpdateAgent = 4,
    DeregisterAgent = 5,
}

impl TryFrom<u8> for AgentMailInstructionDiscriminators {
    type Error = ProgramError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            3 => Ok(Self::RegisterAgent),
            4 => Ok(Self::UpdateAgent),
            5 => Ok(Self::DeregisterAgent),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

/// Marker trait for instruction account structs
///
/// Implementors should use TryFrom<&'a [AccountView]> for parsing
pub trait InstructionAccounts<'a>:
    Sized + TryFrom<&'a [AccountView], Error = ProgramError>
{
}

/// Marker trait for instruction data structs
///
/// Implementors should use TryFrom<&'a [u8]> for parsing
pub trait InstructionData<'a>: Sized + TryFrom<&'a [u8], Error = ProgramError> {
    /// Expected length of instruction data
    const LEN: usize;
}

/// Full instruction combining accounts and data
///
/// Implementors get automatic TryFrom<(&'a [u8], &'a [AccountView])>
pub trait Instruction<'a>: Sized {
    type Accounts: InstructionAccounts<'a>;
    type Data: InstructionData<'a>;

    fn accounts(&self) -> &Self::Accounts;
    fn data(&self) -> &Self::Data;

    /// Parse instruction from data and accounts tuple
    #[inline(always)]
    fn parse(data: &'a [u8], accounts: &'a [AccountView]) -> Result<Self, ProgramError>
    where
        Self: From<(Self::Accounts, Self::Data)>,
    {
        let accounts = Self::Accounts::try_from(accounts)?;
        let data = Self::Data::try_from(data)?;
        Ok(Self::from((accounts, data)))
    }
}
