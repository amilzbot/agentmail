//! # AgentMail Program
//!
//! Decentralized agent-to-agent messaging registry on Solana.
//!
//! ## Features
//! - Agent registry PDA per authority
//! - Register/update/deregister agent endpoints
//! - Name and inbox URL storage
//!
//! ## Architecture
//! Built with Pinocchio (no_std). Clients auto-generated via Codama.

#![no_std]

extern crate alloc;

use pinocchio::address::declare_id;

pub mod errors;
pub mod traits;
pub mod utils;

pub mod instructions;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

declare_id!("AMz2ybwRihFL9X4igLBtqNBEe9qqb4yUvjwNwEaPjNiX");
