# Threat model

## Actors

- **Prover:** may supply malformed statements and witnesses and attempts to produce an accepting proof without a valid short preimage.
- **Verifier integrator:** may accidentally omit public-value or verification-key binding.
- **Network attacker:** may replay proofs, substitute statements, truncate files, or induce resource exhaustion.
- **Parameter attacker:** may advocate weak or trapdoored parameters.
- **Build-chain attacker:** may modify dependencies, toolchains, artifacts, or verification keys.

## Trust boundaries

Private witness data enters the host CLI and SP1 input stream. The public statement is separately known to the verifier and bound by digest. Proof artifacts cross an untrusted transport boundary. The expected SP1 verification key and allowed parameter identifiers are verifier configuration.

## Required invariants

1. The verifier pins the expected program verification key.
2. The verifier recomputes the canonical statement digest.
3. The verifier checks the relation-domain digest.
4. The verifier rejects non-allowlisted parameter identifiers.
5. Proof and statement files are size-limited before deserialization.
6. Production builds are reproducible and dependency-locked.

## Failure consequences

- Missing statement binding: proof substitution across public instances.
- Missing vkey pinning: arbitrary-program proof acceptance.
- Weak parameters: efficient recovery of short preimages despite perfect proof soundness.
- Compromised prover: witness disclosure; proof soundness should remain intact.
- Compromised verifier configuration: systemic false acceptance.
