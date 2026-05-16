//! Core data types for NCP protocol

use serde::{Deserialize, Serialize};

/// Step status in a reasoning chain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepStatus {
    Active,
    Collapsed,
    Fortified,
}

/// Chain status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChainStatus {
    Active,
    Broken,
    Archived,
}

/// A single step in a reasoning chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub index: u32,
    pub content: String,
    pub status: StepStatus,
    pub attack_count: u32,
}

/// A versioned reasoning chain owned by an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chain {
    pub chain_id: String,
    pub agent_id: String,
    pub version: u32,
    pub steps: Vec<Step>,
    pub status: ChainStatus,
    pub created_round: u32,
}

/// An attack record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attack {
    pub attacker_id: String,
    pub target_chain_id: String,
    pub target_step_index: u32,
    pub reason: String,
    pub round: u32,
    pub success: bool,
}

/// Vote on an attack
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Vote {
    Collapsed,
    Defended,
}

/// Result of a voting round
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteResult {
    pub collapse_votes: u32,
    pub defend_votes: u32,
    pub outcome: Vote,
}

impl VoteResult {
    pub fn from_votes(votes: &[Vote]) -> Self {
        let collapse = votes.iter().filter(|v| **v == Vote::Collapsed).count() as u32;
        let defend = votes.iter().filter(|v| **v == Vote::Defended).count() as u32;
        let outcome = if collapse > defend {
            Vote::Collapsed
        } else {
            Vote::Defended
        };
        VoteResult {
            collapse_votes: collapse,
            defend_votes: defend,
            outcome,
        }
    }
}

/// Unique address for any step in the system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StepAddress {
    pub agent_id: String,
    pub version: u32,
    pub step_index: u32,
}

impl std::fmt::Display for StepAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.v{}.Step{}", self.agent_id, self.version, self.step_index)
    }
}
