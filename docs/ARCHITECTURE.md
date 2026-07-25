# Architecture

```text
Public statement                         Private witness
(params, matrix seed, target, context)   short polynomial vector
        |                                      |
        +------------------+-------------------+
                           |
                           v
                  ogunedo-core relation
          canonical checks + matrix expansion + NTT
                           |
                    accept / reject
                           |
                           v
                  Ogunedo SP1 guest
                           |
          commits statement and relation digests
                           |
                           v
              SP1 core/compressed/Groth16 proof
                           |
                           v
                    Ogunedo verifier
       pins program vkey + recomputes statement digest
```

## Trust minimization

Ogunedo uses a mature proving backend rather than implementing a new commitment scheme, FRI protocol, transcript, or recursive verifier. The custom security-critical code is restricted to the relation, canonical encoding, arithmetic, and integration checks.

## Crate boundaries

### `ogunedo-core`

No unsafe code. It is `no_std + alloc` compatible for the guest and exposes the canonical relation to native tests and host tooling.

### `ogunedo-program`

Minimal SP1 entrypoint. It reads the statement and witness, invokes `verify_relation`, and commits one `PublicValues` object.

### `ogunedo-cli`

Handles files, fixture generation, native checks, SP1 execution, proof generation, proof loading, verification-key setup, and statement binding. It refuses unreviewed parameter sets unless explicitly overridden.

## Future Keller layer

The determinant-one Keller compiler should be added as an independent crate only after its exact coordinate semantics, decoder, and test-vector format are frozen. Its output can then be checked by the same SP1 execution layer or used as a distinct public relation.
