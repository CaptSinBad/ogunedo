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

The v1 draft engineering profile is:

```text
parameter_id          = 0x4f470101
R_q                   = Z_12289[X] / (X^256 + 1)
module shape          = 1 x 18
coefficient bound     = |w_i| <= 2
squared norm bound    = ||w||_2^2 <= 10240
primitive generator   = 11
status                = CryptanalysisRequired
```

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
    parameter_digest,
}
```

where

```text
relation_digest = SHA256("OGUNEDO-KISIS-RELATION-V1\0").
```

The guest also commits the canonical parameter digest:

```text
SHA256(
    "OGUNEDO-PARAMETERS-V1\0" ||
    parameter_id_le32 ||
    parameter_name_length_le64 ||
    parameter_name_utf8 ||
    q_le32 ||
    ring_degree_le64 ||
    rows_le64 ||
    columns_le64 ||
    coefficient_bound_i32_le ||
    l2_bound_squared_le64 ||
    primitive_root_le32 ||
    status_code_le32
)
```

Status codes are `0 = DevelopmentOnly`, `1 = CryptanalysisRequired`, and `2 = ProductionApproved`.

The frozen v1 implementation digests are:

```text
relation_digest              = 12ca32d006589f7fa7b3edaa4c1c19d3940661ae8528917738878dfe40d88feb
development_parameter_digest = bbf88444329630d91a6cade2f16681dcdd87eefc59e1b652ee799a1e77da97fe
draft_parameter_digest       = e8c8d261c2d5082a300d241434465245371db7c68b0d5df8e3a05f233af19611
```

## 9. Verification rule

A verifier accepts only when:

1. SP1 verifies the proof under the expected program verification key;
2. the committed protocol and parameter identifiers are expected;
3. the committed statement digest equals the canonical digest of the verifier-supplied statement;
4. the committed relation digest equals the Ogunedo K-ISIS v1 digest;
5. the committed parameter digest equals the locally registered parameter set.

Checking only SP1 proof validity without steps 2-5 is an integration error.

## 10. Keller compiler status

The v1 source tree includes a K-ISIS relation and SP1 guest/verifier integration. It does not yet include a production determinant-one Keller compiler crate. The Keller layer remains a research component and must not be represented as part of the executed SP1 proof relation until a concrete implementation, descriptor format, digest, decoder, and tests are added.

## 11. Versioning

Any change to serialization, matrix expansion, norm semantics, ring arithmetic, or public outputs requires a new protocol version or relation-domain string. Parameter changes require a new parameter identifier.
