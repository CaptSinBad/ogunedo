# Succinct Prover Network proving

This document defines the first paid proving path for Ogunedo through the Succinct Prover Network.

No command in this document should be interpreted as completed unless the corresponding artifact exists and verifies.

The first public benchmark lifecycle has completed for both:

- development compressed mode; and
- production-profile Groth16 mode.

Production Groth16 evidence is recorded in
[`PRODUCTION_GROTH16_EVIDENCE.md`](PRODUCTION_GROTH16_EVIDENCE.md). The proof
artifacts themselves are ignored by Git and should be distributed only through
explicit release evidence bundles.

## Secret handling

Requester wallet material stays local.

- `.env` and `.env.*` are ignored by Git.
- `.env.example` is the only tracked environment file.
- `NETWORK_PRIVATE_KEY` is loaded only by `network-estimate` and `network-prove`.
- The key is never printed, serialized, committed, archived, or sent to GitHub Actions by this repository.
- Do not create a GitHub Actions secret for the requester wallet during the first paid request.

Allowed `.env` keys are:

```text
SP1_PROVER=network
NETWORK_PRIVATE_KEY=
NETWORK_RPC_URL=
```

`NETWORK_RPC_URL` is optional. If absent, the pinned SDK default for mainnet is used.

## Remote-safe fixture

The first network request is locked to:

```text
fixtures/public-benchmark-instance.json
```

The fixture must explicitly contain:

```json
"safe_for_remote_proving": true
```

The fixture's `remote_proving_policy` must state that it contains no trapdoor secret, no signing secret, and no private wallet material. The witness is a deterministic public benchmark witness and may be visible to ordinary network provers.

Network commands refuse every other instance path and every instance without the explicit marker.

## Preflight

Run:

```bash
cargo run --release -p ogunedo-cli -- network-estimate \
  --instance fixtures/public-benchmark-instance.json \
  --mode compressed \
  --max-price-per-pgu <ATOMIC_PROVE_PER_PGU> \
  --output artifacts/network-preflight.json \
  --allow-unreviewed-parameters
```

The command:

- verifies the K-ISIS relation natively;
- confirms the remote-safe fixture marker;
- derives the requester public address without printing the private key;
- builds the pinned SP1 network prover for mainnet;
- derives the vkey;
- executes the guest locally in light mode to obtain cycle and gas limits;
- queries live network proof parameters and requester balance;
- writes a sanitized preflight report.

The preflight report may contain the requester public address, network identifier, RPC hostname, guest ELF hash, vkey, statement digest, relation digest, parameter digest, proof mode, cycle/gas limits, live base fee, max price cap, maximum possible spend, request expiry data, and the exact approval phrase.

It must not contain private keys, mnemonics, wallet JSON, full environment dumps, RPC credentials, or authentication tokens.

## Exact approval

After preflight, stop unless an active runbook explicitly authorizes unattended
submission. Submission requires the exact phrase printed by the command:

```text
APPROVE OGUNEDO NETWORK PROOF UP TO <MAX_PROVE_AMOUNT> PROVE
```

Do not accept "go ahead" or any other vague approval.

## Development compressed request

After exact approval:

```bash
cargo run --release -p ogunedo-cli -- network-prove \
  --instance fixtures/public-benchmark-instance.json \
  --mode compressed \
  --preflight artifacts/network-preflight.json \
  --approval "APPROVE OGUNEDO NETWORK PROOF UP TO <MAX_PROVE_AMOUNT> PROVE" \
  --output proofs/development-compressed.bin \
  --manifest proofs/development-compressed.manifest.json \
  --receipt artifacts/development-network-receipt.json \
  --statement-output artifacts/development-network-statement.json \
  --allow-unreviewed-parameters
```

The command submits the request asynchronously and immediately writes a pending receipt containing the request ID and explorer link before waiting for fulfillment. After the proof is available, it downloads the proof, verifies it immediately, saves it atomically, overwrites the receipt with completed status, then launches a fresh verification subprocess with `NETWORK_PRIVATE_KEY`, `NETWORK_RPC_URL`, and `SP1_PROVER` removed from the environment.

`network-prove` also refuses submission if the sanitized preflight requester balance is below the maximum possible spend recorded in that preflight. Fund and deposit the requester account, rerun `network-estimate`, and use the new exact approval phrase before retrying.

If this request fails, stop before Groth16 and diagnose the exact cause.

## Production Groth16 request

Run a fresh `network-estimate --mode groth16` after the compressed proof succeeds. If any spend value changes, ask for a fresh exact approval phrase.

Only then run `network-prove --mode groth16`.

The first production Groth16 request used:

```text
request ID: 0x6fbb657de5d99b5c7f5bc97d16a7b2c8eadba1767c1463eb992546ab4382cc8b
network: mainnet
proof mode: groth16
proof SHA-256: 84123b23336b9962dec4f0e63602b03156d117b56f5b370f0c86772cc2c8f64d
proof size: 1798 bytes
statement digest: 0x7f72d8f19ef1429a3a9dbd527301815794af4d87460720501835788555535ce1
vkey: 0x00802c99c8f0a957ff88e97091fea717ca15a6f51390b4a721cbb23cee0d81a1
```

Do not submit a second paid request to replace this evidence unless a new
release runbook explicitly authorizes a new request and the previous request is
accounted for.

## Credential-free verification

`ogunedo verify` uses a local verifier path and does not read `.env`. Verification should succeed even when network credentials are absent.

For release evidence, bind the exact artifact bytes and verifier identity:

```bash
unset NETWORK_PRIVATE_KEY NETWORK_RPC_URL SP1_PROVER

ogunedo verify \
  --proof proofs/production-groth16-network.bin \
  --statement artifacts/production-groth16-network-statement.json \
  --expected-proof-sha256 <downloaded-proof-sha256> \
  --expected-proof-size-bytes <downloaded-proof-size> \
  --expected-statement-sha256 <statement-file-sha256> \
  --expected-mode groth16 \
  --expected-vkey <vkey-from-preflight>
```

The hash and size checks are part of the release verification boundary. A file that decodes to a valid SP1 proof object but has extra bytes, missing bytes, or a different hash is not the approved network artifact and must be rejected by release tooling.

## Current limitation

This Windows laptop cannot safely complete expensive SP1 host proving and
verification inside the local memory envelope. The production Groth16 proof was
downloaded locally, but fresh-process verification was moved to the credential-free
Linux VPS after the laptop approached its configured safety boundary. Treat that
resource stop as a local hardware limit, not as a proof failure.
