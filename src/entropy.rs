//! 10-Dimensional Dynamic Entropy Vector
//!
//! Each dimension measures a distinct type of uncertainty (0.0 = certain, 1.0 = chaos).
//! The total entropy is amplified by the impact dimension.

use serde::{Deserialize, Serialize};

/// 10-dimensional entropy vector
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EntropyVector {
    /// Semantic entropy: inter-model understanding consistency
    pub semantic: f64,
    /// Causal entropy: reasoning path uniqueness
    pub causal: f64,
    /// Boundary entropy: conclusion scope clarity
    pub boundary: f64,
    /// Temporal entropy: time sensitivity
    pub temporal: f64,
    /// Dependency entropy: unverified assumption ratio
    pub dependency: f64,
    /// Divergence entropy: inter-model disagreement
    pub divergence: f64,
    /// Information entropy: missing key information
    pub information: f64,
    /// Propagation entropy: error spread range
    pub propagation: f64,
    /// Evidence entropy: evidence reliability
    pub evidence: f64,
    /// Impact entropy: attack shock magnitude (amplifies all others)
    pub impact: f64,
}

impl EntropyVector {
    /// Create a zero entropy vector
    pub fn zero() -> Self {
        Self {
            semantic: 0.0,
            causal: 0.0,
            boundary: 0.0,
            temporal: 0.0,
            dependency: 0.0,
            divergence: 0.0,
            information: 0.0,
            propagation: 0.0,
            evidence: 0.0,
            impact: 0.0,
        }
    }

    /// Compute total entropy with impact amplification
    ///
    /// Formula: H_total = min(1.0, H_mean × (1 + 0.5 × H_impact))
    ///
    /// When impact is high, all uncertainty is amplified.
    pub fn total(&self) -> f64 {
        let values = [
            self.semantic,
            self.causal,
            self.boundary,
            self.temporal,
            self.dependency,
            self.divergence,
            self.information,
            self.propagation,
            self.evidence,
            self.impact,
        ];
        let base_mean = values.iter().sum::<f64>() / values.len() as f64;
        let amplifier = 1.0 + self.impact * 0.5;
        (base_mean * amplifier).min(1.0)
    }

    /// Get the highest entropy dimension
    pub fn max_dimension(&self) -> (&'static str, f64) {
        let dims = self.all_dimensions();
        dims.into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap()
    }

    /// Check if the system can converge
    ///
    /// Convergence requires all dimensions < 0.4 and total < 0.3
    pub fn can_converge(&self) -> bool {
        let all_low = self.all_dimensions().iter().all(|(_, v)| *v < 0.4);
        all_low && self.total() < 0.3
    }

    /// All dimensions as named pairs
    pub fn all_dimensions(&self) -> Vec<(&'static str, f64)> {
        vec![
            ("semantic", self.semantic),
            ("causal", self.causal),
            ("boundary", self.boundary),
            ("temporal", self.temporal),
            ("dependency", self.dependency),
            ("divergence", self.divergence),
            ("information", self.information),
            ("propagation", self.propagation),
            ("evidence", self.evidence),
            ("impact", self.impact),
        ]
    }

    /// Compute delta between two entropy vectors
    pub fn delta(&self, other: &Self) -> Self {
        Self {
            semantic: self.semantic - other.semantic,
            causal: self.causal - other.causal,
            boundary: self.boundary - other.boundary,
            temporal: self.temporal - other.temporal,
            dependency: self.dependency - other.dependency,
            divergence: self.divergence - other.divergence,
            information: self.information - other.information,
            propagation: self.propagation - other.propagation,
            evidence: self.evidence - other.evidence,
            impact: self.impact - other.impact,
        }
    }
}

/// Compute stance-based divergence entropy
///
/// Uses stance vectors instead of text similarity.
/// Each agent's stance is a vector of {-1, 0, +1} over d dimensions.
pub fn compute_divergence(stance_vectors: &[Vec<i8>]) -> f64 {
    let n = stance_vectors.len();
    if n < 2 {
        return 0.5;
    }

    let mut total_agreement = 0.0;
    let mut pair_count = 0;

    for i in 0..n {
        for j in (i + 1)..n {
            let agreement = stance_agreement(&stance_vectors[i], &stance_vectors[j]);
            total_agreement += agreement;
            pair_count += 1;
        }
    }

    if pair_count == 0 {
        return 0.5;
    }

    let avg_agreement = total_agreement / pair_count as f64;
    1.0 - avg_agreement
}

/// Compute agreement between two stance vectors
fn stance_agreement(a: &[i8], b: &[i8]) -> f64 {
    let mut agree_count = 0.0;
    let mut total_count = 0;

    for (va, vb) in a.iter().zip(b.iter()) {
        if *va == 0 && *vb == 0 {
            continue; // Both unmentioned, skip
        }
        total_count += 1;
        if va == vb {
            agree_count += 1.0;
        } else if *va == 0 || *vb == 0 {
            agree_count += 0.5; // One unmentioned = half agreement
        }
    }

    if total_count == 0 {
        0.5
    } else {
        agree_count / total_count as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_total_entropy_zero() {
        let e = EntropyVector::zero();
        assert_eq!(e.total(), 0.0);
    }

    #[test]
    fn test_total_entropy_amplification() {
        let e = EntropyVector {
            impact: 1.0,
            divergence: 0.5,
            ..EntropyVector::zero()
        };
        // base_mean = (0.5 + 1.0) / 10 = 0.15
        // amplified = 0.15 * 1.5 = 0.225
        assert!((e.total() - 0.225).abs() < 0.001);
    }

    #[test]
    fn test_divergence_identical_stances() {
        let stances = vec![vec![1, -1, 1], vec![1, -1, 1], vec![1, -1, 1]];
        let div = compute_divergence(&stances);
        assert!((div - 0.0).abs() < 0.001); // Perfect agreement
    }

    #[test]
    fn test_divergence_opposite_stances() {
        let stances = vec![vec![1, 1, 1], vec![-1, -1, -1]];
        let div = compute_divergence(&stances);
        assert!((div - 1.0).abs() < 0.001); // Complete disagreement
    }
}
