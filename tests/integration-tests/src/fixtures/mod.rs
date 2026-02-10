// Legacy counter fixtures (can be removed once counter tests are cleaned up)
pub mod close_counter;
pub mod create_counter;
pub mod increment;

// AgentMail instruction fixtures
pub mod register_agent;
pub mod update_agent;
pub mod deregister_agent;

pub use close_counter::CloseCounterFixture;
pub use create_counter::CreateCounterFixture;
pub use increment::IncrementFixture;
