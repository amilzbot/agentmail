use crate::traits::Instruction;

use super::{UpdateAgentAccounts, UpdateAgentData};

/// UpdateAgent instruction
///
/// Updates an existing AgentRegistry PDA with new name and inbox_url.
/// Only the authority (agent) can update their own registry.
pub struct UpdateAgent<'a> {
    pub accounts: UpdateAgentAccounts<'a>,
    pub data: UpdateAgentData,
}

impl<'a> Instruction<'a> for UpdateAgent<'a> {
    type Accounts = UpdateAgentAccounts<'a>;
    type Data = UpdateAgentData;

    fn accounts(&self) -> &Self::Accounts {
        &self.accounts
    }

    fn data(&self) -> &Self::Data {
        &self.data
    }
}

impl<'a> From<(UpdateAgentAccounts<'a>, UpdateAgentData)> for UpdateAgent<'a> {
    fn from((accounts, data): (UpdateAgentAccounts<'a>, UpdateAgentData)) -> Self {
        Self { accounts, data }
    }
}

// Unit tests disabled in favor of comprehensive LiteSVM integration tests
#[cfg(disabled_unit_tests)]
mod tests {
    use super::*;
    use pinocchio::{Address, AccountView};
    use alloc::string::ToString;
    use alloc::vec::Vec;
    use core::ptr;

    fn create_mock_account(
        address: Address,
        owner: Address,
        is_signer: bool,
        is_writable: bool,
    ) -> AccountView {
        unsafe { AccountView::new_unchecked(ptr::null_mut()) }
    }

    #[test]
    fn test_update_agent_instruction_creation() {
        let accounts = UpdateAgentAccounts {
            agent_authority: &create_mock_account(
                Address::new_from_array([1u8; 32]),
                Address::new_from_array([11u8; 32]),
                true,
                false,
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

        let data = UpdateAgentData {
            name: "updated-nix".to_string(),
            inbox_url: "https://updated.example.com/inbox".to_string(),
        };

        let instruction = UpdateAgent::from((accounts, data));
        
        assert_eq!(instruction.data().name, "updated-nix");
        assert_eq!(instruction.data().inbox_url, "https://updated.example.com/inbox");
    }

    #[test]
    fn test_update_agent_instruction_parse() {
        let data_bytes = {
            let mut bytes = Vec::new();
            let name = "test-agent";
            let url = "https://test.com/inbox";
            
            bytes.extend_from_slice(&(name.len() as u32).to_le_bytes());
            bytes.extend_from_slice(name.as_bytes());
            bytes.extend_from_slice(&(url.len() as u32).to_le_bytes());
            bytes.extend_from_slice(url.as_bytes());
            
            bytes
        };

        let accounts = [
            create_mock_account(
                Address::new_from_array([1u8; 32]),
                Address::new_from_array([11u8; 32]),
                true,
                false,
            ),
            create_mock_account(
                Address::new_from_array([2u8; 32]),
                crate::ID,
                false,
                true,
            ),
            create_mock_account(
                crate::ID,
                Address::new_from_array([0u8; 32]),
                false,
                false,
            ),
        ];

        let result = UpdateAgent::parse(&data_bytes, &accounts);
        assert!(result.is_ok());
        
        let instruction = result.unwrap();
        assert_eq!(instruction.data().name, "test-agent");
        assert_eq!(instruction.data().inbox_url, "https://test.com/inbox");
    }
}