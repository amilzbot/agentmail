use pinocchio::{account::AccountView, error::ProgramError};

use crate::{
    traits::InstructionAccounts,
    utils::{
        verify_current_program, verify_empty, verify_signer, verify_system_account,
        verify_system_program, verify_writable,
    },
};

/// Accounts for the RegisterAgent instruction
///
/// # Account Layout
/// 0. `[signer, writable]` payer - Pays for account creation
/// 1. `[signer]` agent_authority - Agent's authority (their Solana keypair)
/// 2. `[writable]` agent_registry - Agent registry PDA to be created
/// 3. `[]` system_program - System program for account creation
/// 4. `[]` program - Current program
#[derive(Debug, PartialEq)]
pub struct RegisterAgentAccounts<'a> {
    pub payer: &'a AccountView,
    pub agent_authority: &'a AccountView,
    pub agent_registry: &'a AccountView,
    pub system_program: &'a AccountView,
    pub program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for RegisterAgentAccounts<'a> {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [payer, agent_authority, agent_registry, system_program, program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Payer must be signer and writable (pays for account creation)
        verify_signer(payer)?;
        verify_writable(payer)?;

        // Agent authority must be signer (they own the registry)
        verify_signer(agent_authority)?;

        // Agent registry must be writable, empty, and system-owned (will be created)
        verify_writable(agent_registry)?;
        verify_empty(agent_registry)?;
        verify_system_account(agent_registry)?;

        // Standard system program validation
        verify_system_program(system_program)?;

        // Verify this is our program
        verify_current_program(program)?;

        Ok(Self {
            payer,
            agent_authority,
            agent_registry,
            system_program,
            program,
        })
    }
}

impl<'a> InstructionAccounts<'a> for RegisterAgentAccounts<'a> {}

#[cfg(test)]
mod tests {
    use super::*;
    use pinocchio::{Address, error::ProgramError};
    use core::ptr;
    
    fn create_mock_account(
        address: Address,
        owner: Address,
        lamports: u64,
        data_len: usize,
        is_signer: bool,
        is_writable: bool,
    ) -> AccountView {
        unsafe { AccountView::new_unchecked(ptr::null_mut()) }
    }

    #[test]
    fn test_register_agent_accounts_valid() {
        let payer = create_mock_account(
            Address::new_from_array([1u8; 32]),
            Address::new_from_array([11u8; 32]), // System program
            1000000,
            0,
            true,  // signer
            true,  // writable
        );

        let agent_authority = create_mock_account(
            Address::new_from_array([2u8; 32]),
            Address::new_from_array([11u8; 32]),
            0,
            0,
            true,  // signer
            false, // not writable (just authority)
        );

        let agent_registry = create_mock_account(
            Address::new_from_array([3u8; 32]),
            Address::new_from_array([11u8; 32]), // System program (empty account)
            0,
            0,
            false, // not signer
            true,  // writable
        );

        let system_program = create_mock_account(
            Address::new_from_array([11u8; 32]), // System program ID
            Address::new_from_array([0u8; 32]),  // Native loader
            1,
            0,
            false,
            false,
        );

        let program = create_mock_account(
            crate::ID, // Our program ID from declare_id!
            Address::new_from_array([0u8; 32]), // Native loader
            1,
            0,
            false,
            false,
        );

        let accounts = [payer, agent_authority, agent_registry, system_program, program];
        let result = RegisterAgentAccounts::try_from(&accounts[..]);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_agent_accounts_not_enough_keys() {
        let accounts: [AccountView; 0] = [];
        let result = RegisterAgentAccounts::try_from(&accounts[..]);
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys));
    }

    // Additional validation tests would go here...
}