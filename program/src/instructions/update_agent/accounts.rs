use pinocchio::{account::AccountView, error::ProgramError};

use crate::{
    traits::InstructionAccounts,
    utils::{verify_current_program, verify_signer, verify_writable},
};

/// Accounts for the UpdateAgent instruction
///
/// # Account Layout
/// 0. `[signer]` agent_authority - Agent's authority (must match registry authority)
/// 1. `[writable]` agent_registry - Agent registry PDA to be updated
/// 2. `[]` program - Current program
#[derive(Debug, PartialEq)]
pub struct UpdateAgentAccounts<'a> {
    pub agent_authority: &'a AccountView,
    pub agent_registry: &'a AccountView,
    pub program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for UpdateAgentAccounts<'a> {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [agent_authority, agent_registry, program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Agent authority must be signer (only they can update their registry)
        verify_signer(agent_authority)?;

        // Agent registry must be writable
        verify_writable(agent_registry)?;

        // Verify this is our program
        verify_current_program(program)?;

        Ok(Self {
            agent_authority,
            agent_registry,
            program,
        })
    }
}

impl<'a> InstructionAccounts<'a> for UpdateAgentAccounts<'a> {}

// Unit tests disabled in favor of comprehensive LiteSVM integration tests
#[cfg(disabled_unit_tests)]
mod tests {
    use super::*;
    use core::ptr;
    use pinocchio::{error::ProgramError, Address};

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
    fn test_update_agent_accounts_valid() {
        let agent_authority = create_mock_account(
            Address::new_from_array([1u8; 32]),
            Address::new_from_array([11u8; 32]),
            0,
            0,
            true,  // signer
            false, // not writable (just authority)
        );

        let agent_registry = create_mock_account(
            Address::new_from_array([2u8; 32]),
            crate::ID, // Our program owns this account
            0,
            384,   // Size of AgentRegistry
            false, // not signer
            true,  // writable
        );

        let program = create_mock_account(
            crate::ID,                          // Our program ID from declare_id!
            Address::new_from_array([0u8; 32]), // Native loader
            1,
            0,
            false,
            false,
        );

        let accounts = [agent_authority, agent_registry, program];
        let result = UpdateAgentAccounts::try_from(&accounts[..]);

        assert!(result.is_ok());
    }

    #[test]
    fn test_update_agent_accounts_not_enough_keys() {
        let accounts: [AccountView; 0] = [];
        let result = UpdateAgentAccounts::try_from(&accounts[..]);
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys));
    }

    #[test]
    fn test_update_agent_accounts_authority_not_signer() {
        let agent_authority = create_mock_account(
            Address::new_from_array([1u8; 32]),
            Address::new_from_array([11u8; 32]),
            0,
            0,
            false, // NOT a signer - should fail
            false,
        );

        let agent_registry = create_mock_account(
            Address::new_from_array([2u8; 32]),
            crate::ID,
            0,
            384,
            false,
            true,
        );

        let program = create_mock_account(
            crate::ID,
            Address::new_from_array([0u8; 32]),
            1,
            0,
            false,
            false,
        );

        let accounts = [agent_authority, agent_registry, program];
        let result = UpdateAgentAccounts::try_from(&accounts[..]);

        assert!(result.is_err());
    }

    #[test]
    fn test_update_agent_accounts_registry_not_writable() {
        let agent_authority = create_mock_account(
            Address::new_from_array([1u8; 32]),
            Address::new_from_array([11u8; 32]),
            0,
            0,
            true,
            false,
        );

        let agent_registry = create_mock_account(
            Address::new_from_array([2u8; 32]),
            crate::ID,
            0,
            384,
            false,
            false, // NOT writable - should fail
        );

        let program = create_mock_account(
            crate::ID,
            Address::new_from_array([0u8; 32]),
            1,
            0,
            false,
            false,
        );

        let accounts = [agent_authority, agent_registry, program];
        let result = UpdateAgentAccounts::try_from(&accounts[..]);

        assert!(result.is_err());
    }
}
