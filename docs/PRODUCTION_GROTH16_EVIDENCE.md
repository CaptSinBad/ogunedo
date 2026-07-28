# Production Groth16 evidence

Evidence date: 2026-07-27

This note records the first production-profile Groth16 proof lifecycle for the
public benchmark fixture. It is evidence for the implemented Ogunedo K-ISIS SP1
relation, not a claim that the draft cryptographic parameters are
production-approved.

## Network request

- request ID: `0x6fbb657de5d99b5c7f5bc97d16a7b2c8eadba1767c1463eb992546ab4382cc8b`
- explorer: `https://explorer.succinct.xyz/request/0x6fbb657de5d99b5c7f5bc97d16a7b2c8eadba1767c1463eb992546ab4382cc8b`
- network: `mainnet`
- proof mode: `groth16`
- fixture: `fixtures/public-benchmark-instance.json`
- guest ELF SHA-256: `fd2668813dd45f63e0675d21e1accb60aa928befc7717583e2c3728dcefebd45`
- SP1 vkey: `0x00802c99c8f0a957ff88e97091fea717ca15a6f51390b4a721cbb23cee0d81a1`
- statement digest: `0x7f72d8f19ef1429a3a9dbd527301815794af4d87460720501835788555535ce1`
- relation digest: `0x12ca32d006589f7fa7b3edaa4c1c19d3940661ae8528917738878dfe40d88feb`
- parameter digest: `0xe8c8d261c2d5082a300d241434465245371db7c68b0d5df8e3a05f233af19611`

## Proof artifact

- proof path: `proofs/production-groth16-network.bin`
- proof size: `1,798` bytes
- proof SHA-256: `84123b23336b9962dec4f0e63602b03156d117b56f5b370f0c86772cc2c8f64d`
- statement path: `artifacts/production-groth16-network-statement.json`
- statement SHA-256: `e80c9731bd18dd6cf5aef618e05bbdfb2ed48636d893433c83e6db7c32de6210`

The local `network-prove` process wrote the request ID and saved the downloaded
proof, then entered its fresh credential-free verifier subprocess. That local
fresh verifier was stopped because the 12 GB laptop approached the configured
memory safety boundary. This is recorded as a local resource stop, not as a
cryptographic verification failure.

The downloaded proof was transferred without wallet credentials to the Linux
VPS and verified there.

## Credential-free VPS verification

Clean release build plus verification:

- exit status: `0`
- result: `proof verified and bound to statement`
- elapsed wall time: `40:57.77`
- maximum resident set size: `7,286,616 KiB`
- swaps: `0`
- log: `artifacts/vps-production-groth16-verify-clean.log`

Fresh-process verification with the already-built release binary:

- exit status: `0`
- result: `proof verified and bound to statement`
- elapsed wall time: `1:56.76`
- maximum resident set size: `7,292,540 KiB`
- swaps: `0`
- log: `artifacts/vps-production-groth16-verify-fresh.log`

Both verifier invocations were run with `NETWORK_PRIVATE_KEY`, `NETWORK_RPC_URL`,
and `SP1_PROVER` removed from the process environment.

## Adversarial verification

The production Groth16 adversarial suite passed 18 negative cases:

- mutated proof byte
- truncated proof
- proof with trailing bytes
- empty proof
- proof symlink substitution
- oversized sparse proof
- statement context tamper
- statement target tamper
- statement matrix-seed tamper
- statement protocol-version tamper
- statement parameter-id tamper
- Groth16 proof presented as compressed
- wrong expected verification key
- manifest wrong proof hash
- manifest wrong statement digest
- manifest wrong mode
- manifest wrong verification key
- manifest wrong network

Run result:

- exit status: `0`
- elapsed wall time: `2:25.67`
- maximum resident set size: `7,295,308 KiB`
- swaps: `0`
- machine-readable report: `artifacts/vps-production-groth16-adversarial.json`
- log: `artifacts/vps-production-groth16-adversarial.log`

## Spend accounting

- authorization cap: `5 PROVE`
- pre-submission maximum possible spend: `0.404094000005886633 PROVE`
- postflight maximum possible spend estimate: `0.409165000005886633 PROVE`
- pre-submission requester balance: `9740529999994113367` atomic PROVE
- postflight requester balance: `9336217999988226734` atomic PROVE
- observed balance delta: `404312000005886633` atomic PROVE
- observed balance delta: `0.404312000005886633 PROVE`

The observed balance delta is a before/after requester balance measurement, not
a separate invoice from the network. It remains below the 5 PROVE authorization
cap and below the later postflight maximum estimate.

## Remaining release limits

- GitHub Actions ran successfully on the release-candidate source tag.
- A release-candidate Git tag has been created; GitHub prerelease asset
  publication remains an owner/tooling step.
- The draft parameters remain `CryptanalysisRequired`.
- No `ProductionApproved` parameter profile exists.
- The determinant-one Keller compiler is not implemented in the v1 software.
- Independent audit and public cryptanalysis remain outstanding.
