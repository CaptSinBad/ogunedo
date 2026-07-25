# Security model

## Security goals

Ogunedo K-ISIS aims to provide:

- **knowledge soundness:** a valid proof implies the SP1 program accepted a witness satisfying the registered relation;
- **statement binding:** a proof cannot be replayed as proof for a different public statement without finding a SHA-256 collision or breaking SP1 soundness;
- **witness privacy:** Ogunedo commits no witness data; privacy additionally depends on the selected SP1 proving mode and its documented zero-knowledge guarantees;
- **deterministic semantics:** independent implementations derive the same matrix, target equation, and public digest.

## Assumptions

A deployment depends on:

1. SP1 proof-system soundness and implementation security;
2. collision and second-preimage resistance of SHA-256;
3. hardness of the selected average-case module-ISIS distribution;
4. correct parameter generation and review;
5. secure prover and verifier operations;
6. correct integration of verification-key and statement-digest checks.

Ogunedo does not claim that the registered draft parameter set has completed the analysis required for assumption 3.

## Exact attack surfaces

### Statement substitution

A verifier that checks only the SP1 proof but ignores committed public values can accept a proof for an unintended statement. The CLI performs explicit digest and domain checks before reporting success.

### Verification-key substitution

The expected SP1 program verification key must be pinned by the application. Accepting a caller-supplied verification key allows proof of an arbitrary program.

### Parameter downgrade

Parameter identifiers are digest-bound and committed. Applications must maintain an allowlist and reject development or retired identifiers.

### Non-canonical targets

Every target coefficient must be less than `q`. This avoids alternate encodings of the same residue class and makes the statement digest canonical.

### Length and allocation attacks

The guest resolves a registered parameter set before checking dimensions. Only exact registered lengths are accepted. Production transports should impose message-size limits before deserialization as defense in depth.

### Arithmetic errors

The NTT implementation is compared against schoolbook negacyclic multiplication. `2N | q-1`, primitive-root data, coefficient dimensions, and bounds are registry-controlled.

### Matrix-generation bias

The SHA-256 expander uses rejection sampling rather than direct reduction of a fixed-width word modulo `q`.

### Side channels

The guest relation is not designed as a constant-time native secret-key routine. The proof computation runs over private inputs in the prover environment. Trapdoor generation and preimage sampling are outside this repository and require separate constant-time review.

## Non-goals

This repository does not establish:

- unconditional one-wayness;
- a worst-case-to-average-case reduction for the exact draft module parameters;
- security of a trapdoor sampler;
- resistance of an application protocol that merely embeds this relation;
- audit coverage of Ogunedo itself.
