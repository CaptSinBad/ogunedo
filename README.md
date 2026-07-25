# Ogunedo

**The Ogunedo Proof System** is a proof-of-knowledge framework for short lattice preimages and the production execution layer of the broader Ogunedo/Keller research program.

This repository contains **Ogunedo K-ISIS v1**: an SP1-backed proof that a prover knows a bounded module-lattice witness

\[
  w \in R^k, \qquad A w = u \pmod q,
  \qquad R = \mathbb Z_q[X]/(X^N+1),
\]

without publishing `w`.

The name **Ogunedo** attaches the construction to its inventor, **Ifeanyi Joseph Ogunedo**.

The full determinant-one Keller compiler developed in the accompanying research is a separate layer. This repository proves the underlying K-ISIS relation with a mature zkVM backend; it does not claim that the complete Keller compiler has already been audited or productionized.

## What is production and what is not

The proving backend is pinned to **SP1 6.2.2** and uses its normal program, setup, prove, and verify APIs. The repository does **not** implement a custom FRI, transcript, Merkle tree, Fiat-Shamir transform, or polynomial commitment.

The relation implementation includes:

- canonical public-statement hashing;
- deterministic, rejection-sampled public matrix expansion;
- negacyclic NTT multiplication in `Z_q[X]/(X^N+1)`;
- coefficient and squared-Euclidean-norm bounds;
- exact statement-to-proof binding;
- registered parameter digest binding;
- versioned relation-domain separation;
- negative tests and an independent Python reference model;
- a parameter registry that refuses unreviewed parameters unless the caller opts in.

**Important:** no parameter set in `v0.1.0` is marked `ProductionApproved`. The proof software is production-oriented, but deploying Ogunedo as a cryptographic primitive still requires independent parameter analysis, implementation audit, and operational review. The CLI enforces this distinction.


## Build status

This source tree has passed local `ogunedo-core` Rust tests, clippy, no-default-features compilation, documentation generation, SP1 guest crate checking, the independent Python arithmetic model, deterministic test-vector regeneration, structured-file parsing, and script syntax checks recorded in [`VALIDATION_REPORT.json`](VALIDATION_REPORT.json). The SP1 host CLI and full workspace do not compile on this Windows host because an upstream SP1 JIT dependency requires POSIX APIs; no local SP1 proof is claimed. [`BUILD_STATUS.md`](BUILD_STATUS.md) records the exact commands and blocked gates.

## Repository layout

```text
ogunedo/
|-- crates/ogunedo-core/   # Canonical relation and arithmetic
|-- program/               # SP1 guest program
|-- script/                # Prover/verifier CLI
|-- fixtures/              # Reproducible development instance
|-- docs/                  # Specification and security documents
`-- scripts/               # Reference checks and repository bootstrap
```

## Security statement

For a public statement `S = (params, seed_A, u, context)`, a valid proof attests that the SP1 guest accepted a private witness `w` after checking:

1. the protocol and parameter identifiers are registered;
2. `u` is canonically encoded;
3. `w` has the exact expected dimension;
4. every coefficient of `w` is within the registered bound;
5. `||w||_2^2` is within the registered bound;
6. the deterministic matrix `A = Expand(seed_A)` satisfies `A w = u` in the negacyclic ring;
7. the committed public values contain the canonical digest of `S`, the Ogunedo relation-domain digest, and the registered parameter digest.

The verifier must supply the expected public statement and compare its digest before accepting the proof. The included CLI verifies the SP1 proof first, then performs the explicit digest and domain checks before reporting acceptance.

## Prerequisites

- Rust 1.91.1 or newer compatible toolchain;
- SP1 toolchain corresponding to SP1 6.2.2;
- Go and native build dependencies when generating local Groth16 proofs.

Follow the SP1 installation instructions, then confirm:

```bash
rustc --version
cargo prove --version
```

## Quick start

Run the independent reference implementation:

```bash
python3 scripts/reference_check.py
```

Run native relation tests:

```bash
cargo test -p ogunedo-core
```

Generate a development instance:

```bash
cargo run -p ogunedo-cli -- generate \
  --parameters dev \
  --seed ogunedo-demo \
  --output fixtures/generated-instance.json
```

Check it natively:

```bash
cargo run -p ogunedo-cli -- check \
  --instance fixtures/generated-instance.json
```

Export a shareable statement with the private witness removed:

```bash
cargo run -p ogunedo-cli -- redact \
  --instance fixtures/generated-instance.json \
  --output fixtures/generated-statement.json
```

Execute inside SP1 without proving:

```bash
cargo run --release -p ogunedo-cli -- execute \
  --instance fixtures/generated-instance.json
```

Generate a compressed proof with a development parameter set:

```bash
cargo run --release -p ogunedo-cli -- prove \
  --instance fixtures/generated-instance.json \
  --mode compressed \
  --output proofs/ogunedo-compressed.bin \
  --allow-unreviewed-parameters
```

Verify and bind the saved proof to the statement:

```bash
cargo run --release -p ogunedo-cli -- verify \
  --proof proofs/ogunedo-compressed.bin \
  --statement fixtures/generated-statement.json
```

For a Groth16 proof, use `--mode groth16`.

## Public values

The guest commits only:

```text
protocol_version
parameter_id
SHA256(canonical_public_statement)
SHA256("OGUNEDO-KISIS-RELATION-V1\0")
SHA256(canonical_registered_parameter_set)
```

The private witness is never committed by Ogunedo. The external verifier binds the proof to the expected statement by recomputing the statement digest.

## Reproducibility

Dependencies are exactly pinned where security-sensitive. Before a release:

```bash
cargo update
cargo test -p ogunedo-core
python3 scripts/reference_check.py
cargo run --release -p ogunedo-cli -- execute --instance fixtures/dev-instance.json
```

Commit the resulting `Cargo.lock` and record the SP1 verification-key commitment in the release notes.

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Protocol specification](docs/SPECIFICATION.md)
- [Security model](docs/SECURITY_MODEL.md)
- [One-wayness reduction](docs/ONE_WAYNESS.md)
- [Parameter policy](docs/PARAMETER_POLICY.md)
- [Production-readiness checklist](docs/PRODUCTION_READINESS.md)
- [Threat model](docs/THREAT_MODEL.md)
- [Name and attribution](docs/NAME_AND_ATTRIBUTION.md)
- [Repository bootstrap](docs/REPOSITORY_BOOTSTRAP.md)
- [Implementation audit](docs/IMPLEMENTATION_AUDIT.md)
- [Reproducible builds](docs/REPRODUCIBLE_BUILDS.md)

## License

Dual-licensed under Apache-2.0 or MIT, at your option.
