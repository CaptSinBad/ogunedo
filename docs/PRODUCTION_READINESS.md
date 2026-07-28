# Production-readiness checklist

## Completed in this repository

- [x] Versioned relation and canonical serialization
- [x] Exact statement binding through committed digest
- [x] Versioned relation-domain digest
- [x] Registered dimensions and arithmetic parameters
- [x] Coefficient and squared-norm enforcement
- [x] Rejection-sampled deterministic matrix expansion
- [x] NTT implementation with schoolbook cross-check
- [x] Native negative tests
- [x] Independent Python reference model and test vector
- [x] SP1 guest, host execution, proving, and verification paths
- [x] Core, compressed, and Groth16 proof modes exposed
- [x] Verification-key commitment command
- [x] CLI refusal for unreviewed parameters by default
- [x] CI definitions, dependency policy, and release checklist
- [x] Local laptop resource-safety policy and guarded SP1 runner
- [x] Fail-closed Succinct Prover Network command path
- [x] Public benchmark fixture marked `safe_for_remote_proving=true`
- [x] Exact payment-approval phrase gate before network submission

## Required before real-value deployment

- [x] Reproduce the development compressed SP1 verification path on a clean pinned Linux/VPS environment
- [x] Generate and reload-verify a local development Groth16 proof on the VPS
- [x] Verify the production-profile Groth16 network proof on a clean pinned Linux/VPS environment
- [x] Commit `Cargo.lock` generated from pinned workspace dependencies
- [ ] Reproduce `Cargo.lock` generation on the Linux SP1 CI environment
- [x] Record and independently verify the development compressed program verification-key commitment
- [x] Record and independently verify the production Groth16 release verification key
- [ ] Complete module-ISIS parameter cryptanalysis
- [ ] Add at least one `ProductionApproved` parameter profile
- [ ] Audit `ogunedo-core`, the guest boundary, and verifier integration
- [ ] Run fuzzing and differential testing against a second native implementation
- [ ] Benchmark worst-case proving memory and time
- [ ] Define proof-size, input-size, and rate limits
- [ ] Complete incident response and key/verification-key rotation procedures
- [ ] Obtain an external release sign-off

## Paid network proof gate

The compressed network lifecycle has completed on the VPS evidence path. Groth16 remains open:

- [x] `network-estimate` generated `artifacts/network-preflight.json` from the public benchmark fixture
- [x] Owner supplied the exact approval phrase from the preflight
- [x] Development compressed request submitted to the Succinct Prover Network
- [x] Downloaded compressed proof verified immediately on the VPS
- [x] Downloaded compressed proof verified again in a fresh credential-free VPS process
- [x] Downloaded compressed proof verified with explicit proof hash, proof size, statement hash, proof mode, and vkey bindings
- [x] Compressed proof adversarial/tamper matrix passed
- [x] Production Groth16 preflight regenerated after compressed proof success
- [x] Production Groth16 request submitted under the autonomous 5 PROVE cap
- [x] Production Groth16 proof downloaded and hash-recorded
- [x] Production Groth16 proof verified on the credential-free VPS
- [x] Production Groth16 proof verified again in a fresh credential-free VPS process
- [x] Development Groth16 proof generated locally on the VPS and reload-verified with hash, size, mode, statement, and vkey bindings
- [x] Production Groth16 proof, receipt, evidence manifest, and adversarial-test reports reviewed before any tag

Production Groth16 evidence is recorded in
[`docs/PRODUCTION_GROTH16_EVIDENCE.md`](PRODUCTION_GROTH16_EVIDENCE.md).

## Release gate

A release must not be described as a production cryptographic primitive while any item above remains open. It may be described as a production-oriented proof implementation on an audited proving backend.

Open release blockers as of 2026-07-27:

- GitHub Actions have not yet run on the final candidate commit.
- No `ProductionApproved` parameter profile exists.
- Module-ISIS parameter cryptanalysis remains incomplete.
- Independent implementation audit and public cryptanalysis remain outstanding.
- The GitHub release tag and prerelease have not been created.
