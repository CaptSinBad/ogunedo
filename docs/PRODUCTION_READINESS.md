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

- [ ] Reproduce a full SP1 build and proof on a clean pinned environment
- [x] Commit `Cargo.lock` generated from pinned workspace dependencies
- [ ] Reproduce `Cargo.lock` generation on the Linux SP1 CI environment
- [ ] Record and independently verify the program verification-key commitment
- [ ] Complete module-ISIS parameter cryptanalysis
- [ ] Add at least one `ProductionApproved` parameter profile
- [ ] Audit `ogunedo-core`, the guest boundary, and verifier integration
- [ ] Run fuzzing and differential testing against a second native implementation
- [ ] Benchmark worst-case proving memory and time
- [ ] Define proof-size, input-size, and rate limits
- [ ] Complete incident response and key/verification-key rotation procedures
- [ ] Obtain an external release sign-off

## Paid network proof gate

The network lifecycle is implemented but must not be described as completed until the paid request has actually run:

- [ ] `network-estimate` generated `artifacts/network-preflight.json` from the public benchmark fixture
- [ ] Owner supplied the exact approval phrase from the preflight
- [ ] Development compressed request submitted to the Succinct Prover Network
- [ ] Downloaded compressed proof verified immediately
- [ ] Downloaded compressed proof verified again in a fresh credential-free process
- [ ] Production Groth16 preflight regenerated after compressed proof success
- [ ] Owner supplied a fresh exact approval phrase if the spend changed
- [ ] Production Groth16 request submitted
- [ ] Production Groth16 proof downloaded, saved atomically, reloaded, and verified
- [ ] Proof, receipt, manifest, and adversarial-test reports reviewed before any tag

## Release gate

A release must not be described as a production cryptographic primitive while any item above remains open. It may be described as a production-oriented proof implementation on an audited proving backend.
