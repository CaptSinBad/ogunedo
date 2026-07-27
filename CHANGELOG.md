# Changelog

## 0.1.0 - unreleased

- Define Ogunedo K-ISIS v1 relation and canonical public values.
- Add deterministic rejection-sampled matrix expansion.
- Add NTT and schoolbook negacyclic multiplication.
- Add SP1 6.2.2 guest and CLI proving flows.
- Add development and cryptanalysis-required parameter profiles.
- Add independent Python reference fixture and security documentation.
- Correct the draft v1 squared-norm bound to `10240`.
- Add parameter-digest binding to committed public values.
- Add implementation-audit and reproducible-build status documents.
- Generate and commit `Cargo.lock`.
- Add fail-closed Succinct Prover Network preflight/submission commands guarded by `safe_for_remote_proving=true`, exact approval phrases, sanitized receipts, and credential-free verification.
- Add a deterministic public benchmark fixture for remote proving; proof binaries remain ignored by Git and are distributed only through explicit evidence bundles.
- Record completed production-profile Groth16 network proof evidence for the public benchmark fixture, including request ID, proof hash, statement binding, credential-free VPS verification, and adversarial verifier checks.
