use pinocchio::{account::AccountView, error::ProgramError};

use crate::{
    traits::InstructionAccounts,
    utils::{verify_current_program, verify_signer, verify_writable},
};

/// Accounts for the DeregisterAgent instruction
///
/// # Account Layout
/// 0. `[signer, writable]` agent_authority - Agent's authority (receives reclaimed rent)
/// 1. `[writable]` agent_registry - Agent registry PDA to be closed
/// 2. `[]` program - Current program
pub struct DeregisterAgentAccounts<'a> {
    pub agent_authority: &'a AccountView,
    pub agent_registry: &'a AccountView,
    pub program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for DeregisterAgentAccounts<'a> {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [agent_authority, agent_registry, program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Agent authority must be signer and writable (receives reclaimed rent)
        verify_signer(agent_authority)?;
        verify_writable(agent_authority)?;

        // Agent registry must be writable (will be closed)
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

impl<'a> InstructionAccounts<'a> for DeregisterAgentAccounts<'a> {}

#[cfg(test)]
mod tests {
    use super::*;
    use pinocchio::{Address, Lamports};
    
    fn create_mock_account(
        address: Address,
        owner: Address,
        lamports: Lamports,
        data_len: usize,
        is_signer: bool,
        is_writable: bool,
    ) -> AccountView {
        AccountView::new(
            &address,
            &owner,
            lamports,
            &vec![0u8; data_len],
            is_signer,
            is_writable,
        ).unwrap()
    }

    #[test]
    fn test_deregister_agent_accounts_valid() {
        let agent_authority = create_mock_account(
            Address::new_from_array([1u8; 32]),
            Address::new_from_array([11u8; 32]),
            1000000,
            0,
            true,  // signer
            true,  // writable (will receive rent)
        );

        let agent_registry = create_mock_account(
            Address::new_from_array([2u8; 32]),
            crate::ID, // Our program owns this account
            5000000, // Some rent stored
            384, // Size of AgentRegistry
            false, // not signer
            true,  // writable (will be closed)
        );

        let program = create_mock_account(
            crate::ID, // Our program ID from declare_id!
            Address::new_from_array([0u8; 32]), // Native loader
            1,
            0,
            false,
            false,
        );

        let accounts = [agent_authority, agent_registry, program];
        let result = DeregisterAgentAccounts::try_from(&accounts[..]);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_deregister_agent_accounts_not_enough_keys() {
        let accounts: [AccountView; 0] = [];
        let result = DeregisterAgentAccounts::try_from(&accounts[..]);
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys));
    }

    #[test]
    fn test_deregister_agent_accounts_authority_not_signer() {
        let agent_authority = create_mock_account(
            Address::new_from_array([1u8; 32]),
            Address::new_from_array([11u8; 32]),
            1000000,
            0,
            false, // NOT a signer - should fail
            true,
        );

        let agent_registry = create_mock_account(
            Address::new_from_array([2u8; 32]),
            crate::ID,
            5000000,
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
        let result = DeregisterAgentAccounts::try_from(&accounts[..]);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_deregister_agent_accounts_authority_not_writable() {
        let agent_authority = create_mock_account(
            Address::new_from_array([1u8; 32]),
            Address::new_from_array([11u8; 32]),
            1000000,
            0,
            true,
            false, // NOT writable - should fail
        );

        let agent_registry = create_mock_account(
            Address::new_from_array([2u8; 32]),
            crate::ID,
            5000000,
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
        let result = DeregisterAgentAccounts::try_from(&accounts[..]);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_deregister_agent_accounts_registry_not_writable() {
        let agent_authority = create_mock_account(
            Address::new_from_array([1u8; 32]),
            Address::new_from_array([11u8; 32]),
            1000000,
            0,
            true,
            true,
        );

        let agent_registry = create_mock_account(
            Address::new_from_array([2u8; 32]),
            crate::ID,
            5000000,
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
        let result = DeregisterAgentAccounts::try_from(&accounts[..]);
        
        assert!(result.is_err());
    }
}