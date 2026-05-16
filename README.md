# NCP SDK — Neural Consensus Protocol Core Algorithms

High-performance Rust implementation of the [Neural Consensus Protocol](https://github.com/elbelicojackson-hue/ant-) core algorithms.

## What is NCP?

NCP is a decentralized multi-agent consensus protocol. It defines how independent AI reasoning engines reach verifiable agreement through structured adversarial verification — without any privileged arbiter.

This SDK implements the **pure algorithm layer** — no LLM calls, no network I/O, just math. It can be embedded in any runtime via C-FFI (Python, Node.js, Go, etc.).

## Modules

| Module | Purpose | Key Algorithm |
|--------|---------|---------------|
| `entropy` | 10-dimensional uncertainty measurement | Stance-based divergence, impact amplification |
| `ucb` | Attack target scheduling | UCB1 multi-armed bandit |
| `reputation` | Dynamic agent trust scoring | ELO rating system |
| `consensus` | Verified claim persistence | Time decay + reference boost + challenge propagation |
| `voting` | Decentralized parallel voting | Majority decision + Byzantine fault tolerance |
| `types` | Core data structures | Chain, Step, Attack, Vote, Address |

## Quick Start

```rust
use ncp::{EntropyVector, AttackScheduler, ReputationEngine, ConsensusStore};

// Compute entropy
let entropy = EntropyVector {
    divergence: 0.8,
    impact: 0.5,
    ..EntropyVector::zero()
};
println!("Total entropy: {}", entropy.total()); // Amplified by impact

// Schedule attacks (UCB1)
let mut scheduler = AttackScheduler::new();
scheduler.register_chain("agent_a", 5);
scheduler.register_chain("agent_b", 5);
let targets = scheduler.select_targets("agent_a", 0, 3);
// Returns: [(agent_b, step_1, ∞), ...] — unexplored steps first

// Track reputation (ELO)
let mut rep = ReputationEngine::new();
rep.register("attacker");
rep.register("defender");
rep.record_attack("attacker", "defender", true); // Attack succeeded
println!("Vote weight: {}", rep.vote_weight("attacker")); // > 1.0
```

## Build

```bash
cargo build
cargo test  # 17 tests
```

## Architecture

```
ncp-sdk/
├── Cargo.toml
└── src/
    ├── lib.rs          # Entry point, re-exports
    ├── types.rs        # Chain, Step, Attack, Vote, StepAddress
    ├── entropy.rs      # 10-dim entropy + stance divergence
    ├── ucb.rs          # UCB1 attack scheduler
    ├── reputation.rs   # ELO reputation engine
    ├── consensus.rs    # Consensus store + decay + propagation
    └── voting.rs       # Decentralized voting + Byzantine tolerance
```

## Key Properties

- **O(1) voting latency** — all voters run in parallel regardless of count
- **Byzantine fault tolerant** — tolerates ⌊(n-1)/2⌋ faulty nodes
- **Exploration guaranteed** — UCB1 ensures every step is verified at least once
- **More agents → faster convergence** — coverage scales linearly with participant count

## Mathematical Foundations

See the [NCP Mathematical Foundations PDF](https://github.com/elbelicojackson-hue/ant-/blob/main/paper_v2/NCP-Mathematical-Foundations.pdf) for complete proofs and formulas.

## License

Source Available — See [LICENSE](../LICENSE)
