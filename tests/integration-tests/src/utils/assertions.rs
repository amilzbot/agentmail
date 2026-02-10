use crate::utils::{Address, TestContext};
use agentmail_client::{accounts::AgentRegistry, AGENTMAIL_ID};
use solana_sdk::{instruction::InstructionError, transaction::TransactionError};

pub use agentmail_client::errors::AgentmailError as ProgramError;

pub fn assert_program_error(tx_error: TransactionError, expected: ProgramError) {
    assert_instruction_error(tx_error, InstructionError::Custom(expected as u32));
}

pub fn assert_account_exists(context: &TestContext, address: &Address) {
    let account = context
        .get_account(address)
        .unwrap_or_else(|| panic!("Account {address} should exist"));
    assert!(!account.data.is_empty(), "Account data should not be empty");
}

pub fn assert_account_not_exists(context: &TestContext, address: &Address) {
    assert!(
        context.get_account(address).is_none(),
        "Account {address} should not exist"
    );
}

/// Assert that a transaction error contains the expected instruction error
pub fn assert_instruction_error(tx_error: TransactionError, expected: InstructionError) {
    match tx_error {
        TransactionError::InstructionError(_, err) => {
            assert_eq!(err, expected, "Expected {expected:?}, got {err:?}");
        }
        other => panic!("Expected InstructionError, got {other:?}"),
    }
}

/// Assert that a transaction error is a custom program error with the given code
pub fn assert_custom_error(tx_error: TransactionError, expected_code: u32) {
    assert_instruction_error(tx_error, InstructionError::Custom(expected_code));
}

pub fn assert_agent_registry_account(
    context: &TestContext,
    registry_pda: &Address,
    expected_authority: &Address,
    expected_bump: u8,
    expected_name: &str,
    expected_inbox_url: &str,
) {
    let account = context
        .get_account(registry_pda)
        .expect("Agent registry account should exist");

    assert_eq!(account.owner, AGENTMAIL_ID);

    let registry = AgentRegistry::from_bytes(&account.data).expect("Should deserialize agent registry account");

    assert_eq!(registry.authority.as_ref(), expected_authority.as_ref());
    assert_eq!(registry.bump, expected_bump);
    
    // Parse name from length-prefixed string
    let name_len = u32::from_le_bytes([
        registry.name[0], registry.name[1], registry.name[2], registry.name[3]
    ]) as usize;
    let name_str = std::str::from_utf8(&registry.name[4..4+name_len])
        .expect("Name should be valid UTF-8");
    assert_eq!(name_str, expected_name);
    
    // Parse inbox_url from length-prefixed string
    let url_len = u32::from_le_bytes([
        registry.inbox_url[0], registry.inbox_url[1], registry.inbox_url[2], registry.inbox_url[3]
    ]) as usize;
    let url_str = std::str::from_utf8(&registry.inbox_url[4..4+url_len])
        .expect("Inbox URL should be valid UTF-8");
    assert_eq!(url_str, expected_inbox_url);
}
