use crate::traits::Instruction;

use super::{DeregisterAgentAccounts, DeregisterAgentData};

/// DeregisterAgent instruction
///
/// Closes an existing AgentRegistry PDA and reclaims rent to the authority.
/// Only the authority (agent) can deregister their own registry.
pub struct DeregisterAgent<'a> {
    pub accounts: DeregisterAgentAccounts<'a>,
    pub data: DeregisterAgentData,
}

impl<'a> Instruction<'a> for DeregisterAgent<'a> {
    type Accounts = DeregisterAgentAccounts<'a>;
    type Data = DeregisterAgentData;

    fn accounts(&self) -> &Self::Accounts {
        &self.accounts
    }

    fn data(&self) -> &Self::Data {
        &self.data
    }
}

impl<'a> From<(DeregisterAgentAccounts<'a>, DeregisterAgentData)> for DeregisterAgent<'a> {
    fn from((accounts, data): (DeregisterAgentAccounts<'a>, DeregisterAgentData)) -> Self {
        Self { accounts, data }
    }
}

// Unit tests disabled in favor of comprehensive LiteSVM integration tests
#[cfg(disabled_unit_tests)]
mod tests {
    use super::*;
    use core::ptr;
    use pinocchio::{AccountView, Address};

    fn create_mock_account(
        address: Address,
        owner: Address,
        is_signer: bool,
        is_writable: bool,
    ) -> AccountView {
        unsafe { AccountView::new_unchecked(ptr::null_mut()) }
    }

    #[test]
    fn test_deregister_agent_instruction_creation() {
        let accounts = DeregisterAgentAccounts {
            agent_authority: &create_mock_account(
                Address::new_from_array([1u8; 32]),
                Address::new_from_array([11u8; 32]),
                true,
                true,
            ),
            agent_registry: &create_mock_account(
                Address::new_from_array([2u8; 32]),
                crate::ID,
                false,
                true,
            ),
            program: &create_mock_account(
                crate::ID,
                Address::new_from_array([0u8; 32]),
                false,
                false,
            ),
        };

        let data = DeregisterAgentData;

        let instruction = DeregisterAgent::from((accounts, data));

        // Just verify the instruction was created successfully
        assert_eq!(
            instruction.accounts().agent_authority.address(),
            &Address::new_from_array([1u8; 32])
        );
    }

    #[test]
    fn test_deregister_agent_instruction_parse() {
        let data_bytes = []; // Empty data for deregister

        let accounts = [
            create_mock_account(
                Address::new_from_array([1u8; 32]),
                Address::new_from_array([11u8; 32]),
                true,
                true,
            ),
            create_mock_account(Address::new_from_array([2u8; 32]), crate::ID, false, true),
            create_mock_account(crate::ID, Address::new_from_array([0u8; 32]), false, false),
        ];

        let result = DeregisterAgent::parse(&data_bytes, &accounts);
        assert!(result.is_ok());

        let instruction = result.unwrap();
        assert_eq!(
            instruction.accounts().agent_authority.address(),
            &Address::new_from_array([1u8; 32])
        );
    }
}
