use codama::CodamaInstructions;

/// Instructions for the AgentMail Program.
#[allow(clippy::large_enum_variant)]
#[repr(C, u8)]
#[derive(Clone, Debug, PartialEq, CodamaInstructions)]
pub enum AgentMailInstruction {
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

    /// Update an existing agent registration.
    #[codama(account(name = "agent_authority", signer))]
    #[codama(account(name = "agent_registry", writable))]
    #[codama(account(name = "agentmail_program"))]
    UpdateAgent {
        /// Agent name (UTF-8, max 64 bytes)
        name: alloc::string::String,
        /// Inbox URL (UTF-8, max 256 bytes)
        inbox_url: alloc::string::String,
    } = 4,

    /// Deregister an agent and reclaim rent.
    #[codama(account(name = "agent_authority", signer, writable))]
    #[codama(account(name = "agent_registry", writable))]
    #[codama(account(name = "agentmail_program"))]
    DeregisterAgent {} = 5,
}
