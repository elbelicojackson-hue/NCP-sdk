//! # NCP SDK — Neural Consensus Protocol Core Algorithms
//!
//! High-performance Rust implementation of NCP's core algorithms:
//! - 10-dimensional entropy vector computation
//! - UCB1 attack scheduling
//! - ELO reputation system
//! - Consensus persistence with decay
//! - Decentralized voting logic
//!
//! This crate provides both a native Rust API and C-FFI exports
//! for integration with Python, Node.js, and other languages.

pub mod entropy;
pub mod ucb;
pub mod reputation;
pub mod consensus;
pub mod types;
pub mod voting;

// Re-exports
pub use entropy::EntropyVector;
pub use ucb::AttackScheduler;
pub use reputation::ReputationEngine;
pub use consensus::ConsensusStore;
pub use types::*;
