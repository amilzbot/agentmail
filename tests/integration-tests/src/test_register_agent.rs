use solana_sdk::{signer::Signer, transaction::TransactionError};

use crate::{
    fixtures::register_agent::register_agent,
    utils::{
        pda_utils::find_agent_registry_pda, setup::TestContext, test_helpers::AgentRegistryAccount,
    },
};

#[test]
fn test_register_agent_success() {
    let mut context = TestContext::new();
    let agent_authority = context.create_funded_keypair();
    
    let (agent_registry_pda, bump) = find_agent_registry_pda(&agent_authority.pubkey().into());
    
    let name = "nix".to_string();
    let inbox_url = "https://nix.example.com/inbox".to_string();
    
    let instruction = register_agent(
        &context.payer.pubkey(),
        &agent_authority.pubkey(),
        &agent_registry_pda.into(),
        bump,
        name.clone(),
        inbox_url.clone(),
    );

    let result = context.send_transaction(instruction, &[&agent_authority]);
    assert!(result.is_ok(), "RegisterAgent transaction should succeed");

    // Verify the account was created and has correct data
    let account = context.get_account(&agent_registry_pda.into());
    assert!(account.is_some(), "Agent registry account should exist");

    let registry = AgentRegistryAccount::try_from_account_data(&account.unwrap().data).unwrap();
    assert_eq!(registry.bump, bump);
    assert_eq!(registry.version, 1);
    assert_eq!(registry.authority, agent_authority.pubkey().into());
    assert_eq!(registry.name, name);
    assert_eq!(registry.inbox_url, inbox_url);
    assert!(registry.created_at > 0);
    assert_eq!(registry.created_at, registry.updated_at);
}

#[test]
fn test_register_agent_already_exists() {
    let mut context = TestContext::new();
    let agent_authority = context.create_funded_keypair();
    
    let (agent_registry_pda, bump) = find_agent_registry_pda(&agent_authority.pubkey().into());
    
    let name = "nix".to_string();
    let inbox_url = "https://nix.example.com/inbox".to_string();
    
    // First registration should succeed
    let instruction = register_agent(
        &context.payer.pubkey(),
        &agent_authority.pubkey(),
        &agent_registry_pda.into(),
        bump,
        name.clone(),
        inbox_url.clone(),
    );

    let result = context.send_transaction(instruction, &[&agent_authority]);
    assert!(result.is_ok(), "First RegisterAgent transaction should succeed");

    // Second registration should fail
    let instruction2 = register_agent(
        &context.payer.pubkey(),
        &agent_authority.pubkey(),
        &agent_registry_pda.into(),
        bump,
        "different_name".to_string(),
        "https://different.com/inbox".to_string(),
    );

    let error = context.send_transaction_expect_error(instruction2, &[&agent_authority]);
    // Account already exists error
    assert!(matches!(error, TransactionError::InstructionError(_, _)));
}

#[test]
fn test_register_agent_invalid_authority() {
    let mut context = TestContext::new();
    let agent_authority = context.create_funded_keypair();
    let wrong_authority = context.create_funded_keypair();
    
    let (agent_registry_pda, bump) = find_agent_registry_pda(&agent_authority.pubkey().into());
    
    let instruction = register_agent(
        &context.payer.pubkey(),
        &wrong_authority.pubkey(), // Wrong authority
        &agent_registry_pda.into(), // But PDA derived from correct authority
        bump,
        "nix".to_string(),
        "https://nix.example.com/inbox".to_string(),
    );

    let error = context.send_transaction_expect_error(instruction, &[&wrong_authority]);
    // Should fail with seeds constraint error
    assert!(matches!(error, TransactionError::InstructionError(_, _)));
}

#[test]
fn test_register_agent_name_too_long() {
    let mut context = TestContext::new();
    let agent_authority = context.create_funded_keypair();
    
    let (agent_registry_pda, bump) = find_agent_registry_pda(&agent_authority.pubkey().into());
    
    // Name longer than 64 bytes
    let long_name = "a".repeat(65);
    
    let instruction = register_agent(
        &context.payer.pubkey(),
        &agent_authority.pubkey(),
        &agent_registry_pda.into(),
        bump,
        long_name,
        "https://nix.example.com/inbox".to_string(),
    );

    let error = context.send_transaction_expect_error(instruction, &[&agent_authority]);
    // Should fail with validation error
    assert!(matches!(error, TransactionError::InstructionError(_, _)));
}

#[test]
fn test_register_agent_inbox_url_too_long() {
    let mut context = TestContext::new();
    let agent_authority = context.create_funded_keypair();
    
    let (agent_registry_pda, bump) = find_agent_registry_pda(&agent_authority.pubkey().into());
    
    // URL longer than 256 bytes
    let long_url = format!("https://{}.com/inbox", "a".repeat(250));
    
    let instruction = register_agent(
        &context.payer.pubkey(),
        &agent_authority.pubkey(),
        &agent_registry_pda.into(),
        bump,
        "nix".to_string(),
        long_url,
    );

    let error = context.send_transaction_expect_error(instruction, &[&agent_authority]);
    // Should fail with validation error
    assert!(matches!(error, TransactionError::InstructionError(_, _)));
}

#[test]
fn test_register_agent_empty_fields() {
    let mut context = TestContext::new();
    let agent_authority = context.create_funded_keypair();
    
    let (agent_registry_pda, bump) = find_agent_registry_pda(&agent_authority.pubkey().into());
    
    // Test empty name
    let instruction = register_agent(
        &context.payer.pubkey(),
        &agent_authority.pubkey(),
        &agent_registry_pda.into(),
        bump,
        "".to_string(), // Empty name
        "https://nix.example.com/inbox".to_string(),
    );

    let error = context.send_transaction_expect_error(instruction, &[&agent_authority]);
    assert!(matches!(error, TransactionError::InstructionError(_, _)));
}

#[test] 
fn test_register_agent_invalid_url_format() {
    let mut context = TestContext::new();
    let agent_authority = context.create_funded_keypair();
    
    let (agent_registry_pda, bump) = find_agent_registry_pda(&agent_authority.pubkey().into());
    
    // Test invalid URL (not HTTPS)
    let instruction = register_agent(
        &context.payer.pubkey(),
        &agent_authority.pubkey(),
        &agent_registry_pda.into(),
        bump,
        "nix".to_string(),
        "http://insecure.com/inbox".to_string(), // HTTP instead of HTTPS
    );

    let error = context.send_transaction_expect_error(instruction, &[&agent_authority]);
    assert!(matches!(error, TransactionError::InstructionError(_, _)));
}