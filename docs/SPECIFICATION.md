# Ogunedo K-ISIS v1 specification

## 1. Scope

Ogunedo K-ISIS v1 defines a versioned NP relation and an SP1 program that proves knowledge of a bounded preimage for a public module-lattice equation.

It does not define a trapdoor generator, key-encapsulation mechanism, signature scheme, encryption scheme, or application protocol.

## 2. Algebra

For a registered parameter set, let

- `q` be an odd prime;
- `N` be a power of two with `2N | q-1`;
- `R_q = Z_q[X]/(X^N+1)`;
- `r` be the number of module rows;
- `k` be the number of witness columns.

The public matrix is `A in R_q^(r x k)`, the public target is `u in R_q^r`, and the private witness is a signed coefficient vector representing `w in R^k`.

The relation accepts exactly when

```text
A w = u in R_q^r
```

and the registered coefficient and squared Euclidean norm bounds hold.

## 3. Public statement

```rust
PublicStatement {
    protocol_version: u32,
    parameter_id: u32,
    matrix_seed: [u8; 32],
    target: Vec<u32>,
    context: [u8; 32],
}
```

`context` is application-defined but digest-bound. Applications should use it to bind chain identifiers, contract identifiers, sessions, nonces, or higher-level protocol domains.

## 4. Private witness

```rust
Witness {
    coeffs: Vec<i32>,
}
```

The layout is column-major. Column `j` occupies the interval

```text
[j*N, (j+1)*N)
```

and represents a polynomial in coefficient order.

## 5. Matrix expansion

Each matrix polynomial is derived independently by repeated SHA-256 calls:

```text
SHA256(
    "OGUNEDO-MATRIX-EXPAND-V1\0" ||
    parameter_id_le32 ||
    matrix_seed ||
    row_le32 ||
    column_le32 ||
    block_counter_le64
)
```

The digest is parsed as sixteen little-endian `u16` candidates. Let

```text
L = 65536 - (65536 mod q).
```

A candidate `x` is accepted when `x < L`, and contributes `x mod q`. Rejection sampling removes direct reduction bias. Expansion continues until `N` coefficients have been produced.

## 6. Ring multiplication

The implementation uses a negacyclic NTT.

Let `g` be the registered primitive generator and

```text
psi   = g^((q-1)/(2N)) mod q
omega = psi^2 mod q.
```

Before an ordinary length-`N` cyclic NTT, coefficient `a_i` is multiplied by `psi^i`. After pointwise multiplication and the inverse NTT, coefficient `c_i` is multiplied by `psi^(-i)`. This realizes reduction modulo `X^N+1`.

The test suite compares the NTT result with schoolbook negacyclic multiplication for every registered profile.

## 7. Canonical statement digest

The statement digest is

```text
SHA256(
    "OGUNEDO-STATEMENT-V1\0" ||
    protocol_version_le32 ||
    parameter_id_le32 ||
    matrix_seed ||
    context ||
    target_length_le64 ||
    target[0]_le32 || ... || target[t-1]_le32
).
```

All target coefficients must be strictly less than `q`; non-canonical representatives are rejected.

## 8. Public proof output

The SP1 guest commits a serialized `PublicValues`:

```rust
PublicValues {
    protocol_version,
    parameter_id,
    statement_digest,
    relation_digest,
}
```

where

```text
relation_digest = SHA256("OGUNEDO-KISIS-RELATION-V1\0").
```

## 9. Verification rule

A verifier accepts only when:

1. SP1 verifies the proof under the expected program verification key;
2. the committed protocol and parameter identifiers are expected;
3. the committed statement digest equals the canonical digest of the verifier-supplied statement;
4. the committed relation digest equals the Ogunedo K-ISIS v1 digest.

Checking only SP1 proof validity without steps 2–4 is an integration error.

## 10. Versioning

Any change to serialization, matrix expansion, norm semantics, ring arithmetic, or public outputs requires a new protocol version or relation-domain string. Parameter changes require a new parameter identifier.
