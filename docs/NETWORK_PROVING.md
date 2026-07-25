# Succinct Prover Network proving

This document defines the first paid proving path for Ogunedo through the Succinct Prover Network.

No command in this document should be interpreted as completed unless the corresponding artifact exists and verifies. The current source tree implements the guarded lifecycle; it does not include a downloaded network proof.

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

After preflight, stop. Submission requires the exact phrase printed by the command:

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

The command submits the request asynchronously, waits for fulfillment, downloads the proof, verifies it immediately, saves it atomically, then launches a fresh verification subprocess with `NETWORK_PRIVATE_KEY`, `NETWORK_RPC_URL`, and `SP1_PROVER` removed from the environment.

If this request fails, stop before Groth16 and diagnose the exact cause.

## Production Groth16 request

Run a fresh `network-estimate --mode groth16` after the compressed proof succeeds. If any spend value changes, ask for a fresh exact approval phrase.

Only then run `network-prove --mode groth16`.

## Credential-free verification

`ogunedo verify` uses a local verifier path and does not read `.env`. Verification should succeed even when network credentials are absent.

## Current limitation

This Windows laptop cannot complete the SP1 host CLI compile within the local safety budget, and the full host-side SP1 dependency graph has an upstream POSIX/JIT boundary on this target. Run the network commands on a Linux SP1 environment or CI runner that can compile the host CLI.
