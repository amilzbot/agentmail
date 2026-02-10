use codama::CodamaInstructions;

/// Instructions for the Pinocchio Counter Program.
#[allow(clippy::large_enum_variant)]
#[repr(C, u8)]
#[derive(Clone, Debug, PartialEq, CodamaInstructions)]
pub enum PinocchioCounterInstruction {
    /// Create a new counter for the authority.
    #[codama(account(name = "payer", signer, writable))]
    #[codama(account(name = "authority", signer))]
    #[codama(account(name = "counter", writable))]
    #[codama(account(name = "system_program"))]
    #[codama(account(name = "event_authority"))]
    #[codama(account(name = "pinocchio_counter_program"))]
    CreateCounter {
        /// Bump for the counter PDA
        bump: u8,
    } = 0,

    /// Increment the counter value by 1.
    #[codama(account(name = "authority", signer))]
    #[codama(account(name = "counter", writable))]
    #[codama(account(name = "event_authority"))]
    #[codama(account(name = "pinocchio_counter_program"))]
    Increment {} = 1,

    /// Close the counter and reclaim rent.
    #[codama(account(name = "authority", signer))]
    #[codama(account(name = "counter", writable))]
    #[codama(account(name = "destination", writable))]
    #[codama(account(name = "event_authority"))]
    #[codama(account(name = "pinocchio_counter_program"))]
    CloseCounter {} = 2,

    /// Register an agent in the AgentMail protocol.
    #[codama(account(name = "payer", signer, writable))]
    #[codama(account(name = "agent_authority", signer))]
    #[codama(account(name = "agent_registry", writable))]
    #[codama(account(name = "system_program"))]
    #[codama(account(name = "agentmail_program"))]
    RegisterAgent {
        /// Bump for the agent registry PDA
        bump: u8,
        /// Agent name (UTF-8, max 64 bytes)
        name: alloc::string::String,
        /// Inbox URL (UTF-8, max 256 bytes)
        inbox_url: alloc::string::String,
    } = 3,

    /// Invoked via CPI to emit event data in instruction args (prevents log truncation).
    #[codama(account(name = "event_authority", signer))]
    EmitEvent {} = 228,
}
