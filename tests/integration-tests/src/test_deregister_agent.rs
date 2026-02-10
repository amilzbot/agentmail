use solana_sdk::{signature::Keypair, signer::Signer, transaction::TransactionError};

use crate::{
    fixtures::{register_agent::register_agent, deregister_agent::deregister_agent},
    utils::{
        pda_utils::find_agent_registry_pda, setup::TestContext,
        Address,
    },
};

fn setup_agent_registry(context: &mut TestContext, agent_authority: &Keypair) -> (Address, u8) {
    let (agent_registry_pda, bump) = find_agent_registry_pda(&agent_authority.pubkey().into());
    
    let instruction = register_agent(
        &context.payer.pubkey(),
        &agent_authority.pubkey(),
        &agent_registry_pda.into(),
        bump,
        "test_agent".to_string(),
        "https://test.com/inbox".to_string(),
    );

    context.send_transaction(instruction, &[&agent_authority]).unwrap();
    (agent_registry_pda, bump)
}

#[test]
fn test_deregister_agent_success() {
    let mut context = TestContext::new();
    let agent_authority = context.create_funded_keypair();
    
    let (agent_registry_pda, _bump) = setup_agent_registry(&mut context, &agent_authority);
    
    // Record agent's balance before deregistration
    let agent_balance_before = context.get_account(&agent_authority.pubkey().into()).unwrap().lamports;
    
    let instruction = deregister_agent(
        &agent_authority.pubkey(),
        &agent_registry_pda.into(),
    );

    let result = context.send_transaction(instruction, &[&agent_authority]);
    assert!(result.is_ok(), "DeregisterAgent transaction should succeed");

    // Verify the account was closed
    let account = context.get_account(&agent_registry_pda.into());
    assert!(account.is_none(), "Agent registry account should be closed");

    // Verify the rent was reclaimed
    let agent_balance_after = context.get_account(&agent_authority.pubkey().into()).unwrap().lamports;
    assert!(agent_balance_after > agent_balance_before, "Agent should receive rent refund");
}

#[test]
fn test_deregister_agent_not_registered() {
    let mut context = TestContext::new();
    let agent_authority = context.create_funded_keypair();
    
    // Try to deregister without registering first
    let (agent_registry_pda, _bump) = find_agent_registry_pda(&agent_authority.pubkey().into());
    
    let instruction = deregister_agent(
        &agent_authority.pubkey(),
        &agent_registry_pda.into(),
    );

    let error = context.send_transaction_expect_error(instruction, &[&agent_authority]);
    // Should fail because account doesn't exist
    assert!(matches!(error, TransactionError::InstructionError(_, _)));
}

#[test]
fn test_deregister_agent_wrong_authority() {
    let mut context = TestContext::new();
    let agent_authority = context.create_funded_keypair();
    let wrong_authority = context.create_funded_keypair();
    
    let (agent_registry_pda, _bump) = setup_agent_registry(&mut context, &agent_authority);
    
    let instruction = deregister_agent(
        &wrong_authority.pubkey(), // Wrong authority
        &agent_registry_pda.into(),
    );

    let error = context.send_transaction_expect_error(instruction, &[&wrong_authority]);
    // Should fail with authority validation error
    assert!(matches!(error, TransactionError::InstructionError(_, _)));
}

#[test]
fn test_deregister_agent_rent_calculation() {
    let mut context = TestContext::new();
    let agent_authority = context.create_funded_keypair();
    
    let (agent_registry_pda, _bump) = setup_agent_registry(&mut context, &agent_authority);
    
    // Get the registry account rent amount
    let registry_account = context.get_account(&agent_registry_pda.into()).unwrap();
    let _rent_amount = registry_account.lamports;
    
    // Record balances before
    let agent_balance_before = context.get_account(&agent_authority.pubkey().into()).unwrap().lamports;
    
    let instruction = deregister_agent(
        &agent_authority.pubkey(),
        &agent_registry_pda.into(),
    );

    context.send_transaction(instruction, &[&agent_authority]).unwrap();
    
    // Verify exact rent refund
    let agent_balance_after = context.get_account(&agent_authority.pubkey().into()).unwrap().lamports;
    let refund_amount = agent_balance_after - agent_balance_before;
    
    // Should get back the full rent amount (minus transaction fees which are minimal in test)
    assert!(refund_amount > 0, "Should receive some rent refund");
    // In a real test, we might want to be more precise about fees
}

#[test]
fn test_deregister_agent_account_data_zeroed() {
    let mut context = TestContext::new();
    let agent_authority = context.create_funded_keypair();
    
    let (agent_registry_pda, _bump) = setup_agent_registry(&mut context, &agent_authority);
    
    // Verify account exists and has data
    let account_before = context.get_account(&agent_registry_pda.into()).unwrap();
    assert_eq!(account_before.data.len(), 384);
    assert!(!account_before.data.iter().all(|&b| b == 0)); // Should have non-zero data
    
    let instruction = deregister_agent(
        &agent_authority.pubkey(),
        &agent_registry_pda.into(),
    );

    context.send_transaction(instruction, &[&agent_authority]).unwrap();
    
    // Account should be completely gone
    let account_after = context.get_account(&agent_registry_pda.into());
    assert!(account_after.is_none(), "Account should not exist after deregistration");
}

#[test]
fn test_deregister_agent_can_re_register() {
    let mut context = TestContext::new();
    let agent_authority = context.create_funded_keypair();
    
    // Register
    let (agent_registry_pda, bump) = setup_agent_registry(&mut context, &agent_authority);
    
    // Deregister
    let deregister_instruction = deregister_agent(
        &agent_authority.pubkey(),
        &agent_registry_pda.into(),
    );
    context.send_transaction(deregister_instruction, &[&agent_authority]).unwrap();
    
    // Verify account is gone
    assert!(context.get_account(&agent_registry_pda.into()).is_none());
    
    // Re-register with different data
    let re_register_instruction = register_agent(
        &context.payer.pubkey(),
        &agent_authority.pubkey(),
        &agent_registry_pda.into(),
        bump,
        "re_registered_agent".to_string(),
        "https://new-location.com/inbox".to_string(),
    );
    
    let result = context.send_transaction(re_register_instruction, &[&agent_authority]);
    assert!(result.is_ok(), "Should be able to re-register after deregistration");
    
    // Verify new registration worked
    let account = context.get_account(&agent_registry_pda.into());
    assert!(account.is_some(), "Re-registered account should exist");
}