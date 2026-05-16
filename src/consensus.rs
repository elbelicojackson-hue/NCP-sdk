//! Consensus Persistence with Decay
//!
//! Stores verified claims with time-based decay, cross-session referencing,
//! and dependency propagation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

const DECAY_RATE: f64 = 0.005;      // Per day
const DECAY_FLOOR: f64 = 0.3;
const REFERENCE_BOOST: f64 = 0.02;
const CHALLENGE_PENALTY: f64 = 0.15;

/// A verified consensus claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedClaim {
    pub claim_id: String,
    pub content: String,
    pub alpha: f64,
    pub confidence: f64,
    pub verified_at: u64,       // Unix timestamp
    pub session_id: String,
    pub participants: Vec<String>,
    pub agree_count: u32,
    pub total_count: u32,
    pub attacks_survived: u32,
    pub reference_count: u32,
    pub status: ClaimStatus,
    pub depends_on: Vec<String>,
    pub depended_by: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimStatus {
    Active,
    Decayed,
    Challenged,
    Revoked,
}

/// Consensus persistence store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusStore {
    claims: HashMap<String, VerifiedClaim>,
}

impl ConsensusStore {
    pub fn new() -> Self {
        Self {
            claims: HashMap::new(),
        }
    }

    /// Store a new verified claim
    pub fn store(&mut self, claim: VerifiedClaim) -> String {
        let id = claim.claim_id.clone();
        // Merge if exists (take higher alpha)
        if let Some(existing) = self.claims.get_mut(&id) {
            if claim.alpha > existing.alpha {
                existing.alpha = claim.alpha;
                existing.attacks_survived += claim.attacks_survived;
                existing.confidence = (existing.confidence + 0.1).min(1.0);
            }
        } else {
            self.claims.insert(id.clone(), claim);
        }
        id
    }

    /// Reference a claim (boosts confidence)
    pub fn reference(&mut self, claim_id: &str) -> Option<&VerifiedClaim> {
        if let Some(claim) = self.claims.get_mut(claim_id) {
            if claim.status == ClaimStatus::Revoked {
                return None;
            }
            claim.reference_count += 1;
            claim.confidence = (claim.confidence + REFERENCE_BOOST).min(1.0);
            Some(claim)
        } else {
            None
        }
    }

    /// Challenge a claim
    pub fn challenge(&mut self, claim_id: &str, success: bool) -> bool {
        let dependents = {
            let claim = match self.claims.get_mut(claim_id) {
                Some(c) => c,
                None => return false,
            };

            if success {
                claim.confidence -= CHALLENGE_PENALTY;
                if claim.confidence <= 0.1 {
                    claim.status = ClaimStatus::Revoked;
                    claim.depended_by.clone()
                } else {
                    vec![]
                }
            } else {
                claim.attacks_survived += 1;
                claim.confidence = (claim.confidence + 0.05).min(1.0);
                vec![]
            }
        };

        // Propagate revocation to dependents
        for dep_id in &dependents {
            if let Some(dep) = self.claims.get_mut(dep_id) {
                dep.confidence -= 0.2;
                if dep.confidence <= 0.2 {
                    dep.status = ClaimStatus::Challenged;
                }
            }
        }

        true
    }

    /// Apply time decay to all claims
    pub fn apply_decay(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        for claim in self.claims.values_mut() {
            if claim.status == ClaimStatus::Revoked {
                continue;
            }

            let age_days = (now - claim.verified_at) as f64 / 86400.0;
            let decayed = 1.0 - DECAY_RATE * age_days
                + REFERENCE_BOOST * claim.reference_count as f64;
            claim.confidence = decayed.max(DECAY_FLOOR).min(1.0);

            if claim.confidence <= DECAY_FLOOR && claim.status == ClaimStatus::Active {
                claim.status = ClaimStatus::Decayed;
            }
        }
    }

    /// Query claims by minimum alpha and confidence
    pub fn query(&self, min_alpha: f64, min_confidence: f64, limit: usize) -> Vec<&VerifiedClaim> {
        let mut results: Vec<&VerifiedClaim> = self
            .claims
            .values()
            .filter(|c| {
                c.status != ClaimStatus::Revoked
                    && c.alpha >= min_alpha
                    && c.confidence >= min_confidence
            })
            .collect();

        results.sort_by(|a, b| {
            (b.alpha * b.confidence)
                .partial_cmp(&(a.alpha * a.confidence))
                .unwrap()
        });
        results.truncate(limit);
        results
    }

    /// Get store statistics
    pub fn stats(&self) -> StoreStats {
        let active = self.claims.values().filter(|c| c.status == ClaimStatus::Active).count();
        let decayed = self.claims.values().filter(|c| c.status == ClaimStatus::Decayed).count();
        let revoked = self.claims.values().filter(|c| c.status == ClaimStatus::Revoked).count();
        let total_refs: u32 = self.claims.values().map(|c| c.reference_count).sum();

        StoreStats {
            total: self.claims.len(),
            active,
            decayed,
            revoked,
            total_references: total_refs,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreStats {
    pub total: usize,
    pub active: usize,
    pub decayed: usize,
    pub revoked: usize,
    pub total_references: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_claim(id: &str, alpha: f64) -> VerifiedClaim {
        VerifiedClaim {
            claim_id: id.to_string(),
            content: format!("Test claim {}", id),
            alpha,
            confidence: 1.0,
            verified_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            session_id: "test".to_string(),
            participants: vec!["a".to_string(), "b".to_string()],
            agree_count: 2,
            total_count: 3,
            attacks_survived: 0,
            reference_count: 0,
            status: ClaimStatus::Active,
            depends_on: vec![],
            depended_by: vec![],
        }
    }

    #[test]
    fn test_store_and_query() {
        let mut store = ConsensusStore::new();
        store.store(make_claim("c1", 0.8));
        store.store(make_claim("c2", 0.5));

        let results = store.query(0.6, 0.5, 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].claim_id, "c1");
    }

    #[test]
    fn test_challenge_revokes() {
        let mut store = ConsensusStore::new();
        store.store(make_claim("c1", 0.8));

        // Challenge 7 times to revoke (7 × 0.15 = 1.05 > 1.0)
        for _ in 0..7 {
            store.challenge("c1", true);
        }

        let claim = store.claims.get("c1").unwrap();
        assert_eq!(claim.status, ClaimStatus::Revoked);
    }

    #[test]
    fn test_reference_boosts() {
        let mut store = ConsensusStore::new();
        let mut claim = make_claim("c1", 0.8);
        claim.confidence = 0.5;
        store.store(claim);

        store.reference("c1");
        store.reference("c1");

        let c = store.claims.get("c1").unwrap();
        assert!((c.confidence - 0.54).abs() < 0.01); // 0.5 + 2×0.02
    }
}
