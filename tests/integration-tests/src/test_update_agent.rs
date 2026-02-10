use solana_sdk::{signature::Keypair, signer::Signer, transaction::TransactionError};

use crate::{
    fixtures::{register_agent::register_agent, update_agent::update_agent},
    utils::{
        pda_utils::find_agent_registry_pda, setup::TestContext, test_helpers::AgentRegistryAccount,
        Address,
    },
};

fn setup_agent_registry(context: &mut TestContext, agent_authority: &Keypair) -> (Address, u8) {
    let (agent_registry_pda, bump) = find_agent_registry_pda(&agent_authority.pubkey());

    let instruction = register_agent(
        &context.payer.pubkey(),
        &agent_authority.pubkey(),
        &agent_registry_pda,
        bump,
        "original_name".to_string(),
        "https://original.com/inbox".to_string(),
    );

    context
        .send_transaction(instruction, &[agent_authority])
        .unwrap();
    (agent_registry_pda, bump)
}

#[test]
fn test_update_agent_success() {
    let mut context = TestContext::new();
    let agent_authority = context.create_funded_keypair();

    let (agent_registry_pda, _bump) = setup_agent_registry(&mut context, &agent_authority);

    // Wait a bit to ensure different timestamp
    context.warp_to_next_slot();

    let new_name = "updated_name".to_string();
    let new_inbox_url = "https://updated.com/inbox".to_string();

    let instruction = update_agent(
        &agent_authority.pubkey(),
        &agent_registry_pda,
        new_name.clone(),
        new_inbox_url.clone(),
    );

    let result = context.send_transaction(instruction, &[&agent_authority]);
    assert!(result.is_ok(), "UpdateAgent transaction should succeed");

    // Verify the account was updated
    let account = context.get_account(&agent_registry_pda);
    assert!(
        account.is_some(),
        "Agent registry account should still exist"
    );

    let registry = AgentRegistryAccount::try_from_account_data(&account.unwrap().data).unwrap();
    assert_eq!(registry.authority, agent_authority.pubkey());
    assert_eq!(registry.name, new_name);
    assert_eq!(registry.inbox_url, new_inbox_url);
    assert!(
        registry.updated_at > registry.created_at,
        "updated_at should be newer than created_at"
    );
}

#[test]
fn test_update_agent_not_registered() {
    let mut context = TestContext::new();
    let agent_authority = context.create_funded_keypair();

    // Try to update without registering first
    let (agent_registry_pda, _bump) = find_agent_registry_pda(&agent_authority.pubkey());

    let instruction = update_agent(
        &agent_authority.pubkey(),
        &agent_registry_pda,
        "new_name".to_string(),
        "https://new.com/inbox".to_string(),
    );

    let error = context.send_transaction_expect_error(instruction, &[&agent_authority]);
    // Should fail because account doesn't exist
    assert!(matches!(error, TransactionError::InstructionError(_, _)));
}

#[test]
fn test_update_agent_wrong_authority() {
    let mut context = TestContext::new();
    let agent_authority = context.create_funded_keypair();
    let wrong_authority = context.create_funded_keypair();

    let (agent_registry_pda, _bump) = setup_agent_registry(&mut context, &agent_authority);

    let instruction = update_agent(
        &wrong_authority.pubkey(), // Wrong authority
        &agent_registry_pda,
        "hacked_name".to_string(),
        "https://hacker.com/inbox".to_string(),
    );

    let error = context.send_transaction_expect_error(instruction, &[&wrong_authority]);
    // Should fail with authority validation error
    assert!(matches!(error, TransactionError::InstructionError(_, _)));
}

#[test]
fn test_update_agent_name_too_long() {
    let mut context = TestContext::new();
    let agent_authority = context.create_funded_keypair();

    let (agent_registry_pda, _bump) = setup_agent_registry(&mut context, &agent_authority);

    // Name longer than 64 bytes
    let long_name = "a".repeat(65);

    let instruction = update_agent(
        &agent_authority.pubkey(),
        &agent_registry_pda,
        long_name,
        "https://valid.com/inbox".to_string(),
    );

    let error = context.send_transaction_expect_error(instruction, &[&agent_authority]);
    // Should fail with validation error
    assert!(matches!(error, TransactionError::InstructionError(_, _)));
}

#[test]
fn test_update_agent_inbox_url_too_long() {
    let mut context = TestContext::new();
    let agent_authority = context.create_funded_keypair();

    let (agent_registry_pda, _bump) = setup_agent_registry(&mut context, &agent_authority);

    // URL longer than 256 bytes
    let long_url = format!("https://{}.com/inbox", "a".repeat(250));

    let instruction = update_agent(
        &agent_authority.pubkey(),
        &agent_registry_pda,
        "valid_name".to_string(),
        long_url,
    );

    let error = context.send_transaction_expect_error(instruction, &[&agent_authority]);
    // Should fail with validation error
    assert!(matches!(error, TransactionError::InstructionError(_, _)));
}

#[test]
fn test_update_agent_empty_name() {
    let mut context = TestContext::new();
    let agent_authority = context.create_funded_keypair();

    let (agent_registry_pda, _bump) = setup_agent_registry(&mut context, &agent_authority);

    let instruction = update_agent(
        &agent_authority.pubkey(),
        &agent_registry_pda,
        "".to_string(), // Empty name
        "https://valid.com/inbox".to_string(),
    );

    let error = context.send_transaction_expect_error(instruction, &[&agent_authority]);
    // Should fail with validation error
    assert!(matches!(error, TransactionError::InstructionError(_, _)));
}

#[test]
fn test_update_agent_invalid_url_format() {
    let mut context = TestContext::new();
    let agent_authority = context.create_funded_keypair();

    let (agent_registry_pda, _bump) = setup_agent_registry(&mut context, &agent_authority);

    let instruction = update_agent(
        &agent_authority.pubkey(),
        &agent_registry_pda,
        "valid_name".to_string(),
        "http://insecure.com/inbox".to_string(), // HTTP instead of HTTPS
    );

    let error = context.send_transaction_expect_error(instruction, &[&agent_authority]);
    // Should fail with validation error
    assert!(matches!(error, TransactionError::InstructionError(_, _)));
}

#[test]
fn test_update_agent_preserves_other_fields() {
    let mut context = TestContext::new();
    let agent_authority = context.create_funded_keypair();

    let (agent_registry_pda, bump) = setup_agent_registry(&mut context, &agent_authority);

    // Get original data
    let original_account = context.get_account(&agent_registry_pda).unwrap();
    let original_registry =
        AgentRegistryAccount::try_from_account_data(&original_account.data).unwrap();

    // Update with same values to ensure other fields are preserved
    let instruction = update_agent(
        &agent_authority.pubkey(),
        &agent_registry_pda,
        "new_name".to_string(),
        "https://new.com/inbox".to_string(),
    );

    context
        .send_transaction(instruction, &[&agent_authority])
        .unwrap();

    // Verify preserved fields
    let updated_account = context.get_account(&agent_registry_pda).unwrap();
    let updated_registry =
        AgentRegistryAccount::try_from_account_data(&updated_account.data).unwrap();

    assert_eq!(updated_registry.bump, bump);
    assert_eq!(updated_registry.version, original_registry.version);
    assert_eq!(updated_registry.authority, original_registry.authority);
    assert_eq!(updated_registry.created_at, original_registry.created_at);
    // But updated_at should have changed
    assert!(updated_registry.updated_at >= original_registry.updated_at);
}
