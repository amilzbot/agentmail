use solana_sdk::instruction::InstructionError;

use crate::utils::{assert_instruction_error, Address, TestContext};

use super::traits::InstructionTestFixture;

pub const RANDOM_ADDRESS: Address =
    Address::from_str_const("EpkG1ek8zrHWHqgUv42fTd6vJPsceSzkPSZfGaoLUGqf");

/// Test that removing a required signer fails with MissingRequiredSignature
///
/// # Arguments
/// * `ctx` - Test context
/// * `account_index` - Index in instruction.accounts for the signer
/// * `signer_vec_index` - Index into the signers Vec to remove
pub fn test_missing_signer<T: InstructionTestFixture>(
    ctx: &mut TestContext,
    account_index: usize,
    signer_vec_index: usize,
) {
    let error = T::build_valid(ctx)
        .without_signer(account_index, signer_vec_index)
        .send_expect_error(ctx);
    assert_instruction_error(error, InstructionError::MissingRequiredSignature);
}

/// Test that making a required writable account read-only fails
///
/// # Arguments
/// * `ctx` - Test context
/// * `account_index` - Index in instruction.accounts that should be writable
pub fn test_not_writable<T: InstructionTestFixture>(ctx: &mut TestContext, account_index: usize) {
    let error = T::build_valid(ctx)
        .with_readonly_at(account_index)
        .send_expect_error(ctx);
    assert_instruction_error(error, InstructionError::Immutable);
}

/// Test that providing the wrong system program fails
pub fn test_wrong_system_program<T: InstructionTestFixture>(ctx: &mut TestContext) {
    let index = T::system_program_index().expect("Instruction must have system_program_index");
    let error = T::build_valid(ctx)
        .with_account_at(index, RANDOM_ADDRESS)
        .send_expect_error(ctx);
    assert_instruction_error(error, InstructionError::IncorrectProgramId);
}

/// Test that providing the wrong current program fails
pub fn test_wrong_current_program<T: InstructionTestFixture>(ctx: &mut TestContext) {
    let index = T::current_program_index().expect("Instruction must have current_program_index");
    let error = T::build_valid(ctx)
        .with_account_at(index, RANDOM_ADDRESS)
        .send_expect_error(ctx);
    assert_instruction_error(error, InstructionError::IncorrectProgramId);
}

/// Test that providing a wrong account at a given index fails
///
/// # Arguments
/// * `ctx` - Test context
/// * `account_index` - Index in instruction.accounts to replace
/// * `expected_error` - The expected instruction error
pub fn test_wrong_account<T: InstructionTestFixture>(
    ctx: &mut TestContext,
    account_index: usize,
    expected_error: InstructionError,
) {
    let error = T::build_valid(ctx)
        .with_account_at(account_index, Address::new_unique())
        .send_expect_error(ctx);
    assert_instruction_error(error, expected_error);
}

/// Test that empty instruction data fails
pub fn test_empty_data<T: InstructionTestFixture>(ctx: &mut TestContext) {
    let error = T::build_valid(ctx).with_data_len(0).send_expect_error(ctx);
    assert_instruction_error(error, InstructionError::InvalidInstructionData);
}

/// Test that truncated instruction data fails
pub fn test_truncated_data<T: InstructionTestFixture>(ctx: &mut TestContext) {
    let expected_len = T::data_len();
    if expected_len > 1 {
        let error = T::build_valid(ctx)
            .with_data_len(expected_len - 1)
            .send_expect_error(ctx);
        assert_instruction_error(error, InstructionError::InvalidInstructionData);
    }
}

/// Test that providing an invalid bump for a PDA fails
///
/// # Arguments
/// * `ctx` - Test context
/// * `bump_byte_index` - Index of the bump byte in instruction data
/// * `invalid_bump` - A bump value that won't match the PDA derivation
pub fn test_invalid_bump<T: InstructionTestFixture>(
    ctx: &mut TestContext,
    bump_byte_index: usize,
    invalid_bump: u8,
) {
    let error = T::build_valid(ctx)
        .with_data_byte_at(bump_byte_index, invalid_bump)
        .send_expect_error(ctx);
    assert_instruction_error(error, InstructionError::InvalidSeeds);
}

/// Helper struct for parsing AgentRegistry account data in tests
#[derive(Debug, Clone, PartialEq)]
pub struct AgentRegistryAccount {
    pub bump: u8,
    pub version: u8,
    pub authority: Address,
    pub name: String,
    pub inbox_url: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl AgentRegistryAccount {
    pub fn try_from_account_data(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < 318 {  // 1 + 1 + 6 + 32 + 68 + 260 + 8 + 8 = 384 total, but discriminator is separate
            return Err("Invalid account data length");
        }

        let bump = data[0];
        let version = data[1];
        // Skip padding bytes 2-7
        
        // Authority is at offset 8, 32 bytes
        let authority = Address::from(<[u8; 32]>::try_from(&data[8..40]).unwrap());
        
        // Name is at offset 40, fixed 68 bytes (4 bytes len + up to 64 bytes data)
        let name_len = u32::from_le_bytes([data[40], data[41], data[42], data[43]]) as usize;
        if name_len > 64 {
            return Err("Invalid name length");
        }
        let name_bytes = &data[44..44 + name_len];
        let name = String::from_utf8(name_bytes.to_vec()).map_err(|_| "Invalid name UTF-8")?;
        
        // Inbox URL is at offset 108 (40 + 68), fixed 260 bytes (4 bytes len + up to 256 bytes data)
        let url_len = u32::from_le_bytes([data[108], data[109], data[110], data[111]]) as usize;
        if url_len > 256 {
            return Err("Invalid inbox_url length");
        }
        let url_bytes = &data[112..112 + url_len];
        let inbox_url = String::from_utf8(url_bytes.to_vec()).map_err(|_| "Invalid inbox_url UTF-8")?;
        
        // Timestamps are at offset 368 and 376 (8 bytes each)
        let created_at = i64::from_le_bytes([
            data[368], data[369], data[370], data[371],
            data[372], data[373], data[374], data[375],
        ]);
        let updated_at = i64::from_le_bytes([
            data[376], data[377], data[378], data[379],
            data[380], data[381], data[382], data[383],
        ]);

        Ok(Self {
            bump,
            version,
            authority,
            name,
            inbox_url,
            created_at,
            updated_at,
        })
    }
}
