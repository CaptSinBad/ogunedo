#!/usr/bin/env python3
"""Independent Python reference model for Ogunedo K-ISIS v1.

This script does not generate a proof. It independently checks the deterministic
matrix expander, negacyclic NTT multiplication, relation equation, and canonical
digest used by the Rust/SP1 implementation.
"""

from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MATRIX_DOMAIN = b"OGUNEDO-MATRIX-EXPAND-V1\0"
STATEMENT_DOMAIN = b"OGUNEDO-STATEMENT-V1\0"
RELATION_DOMAIN = b"OGUNEDO-KISIS-RELATION-V1\0"
PROTOCOL_VERSION = 1
DEV_PARAMETERS_ID = 0x4F470001


@dataclass(frozen=True)
class Parameters:
    identifier: int
    q: int
    n: int
    rows: int
    columns: int
    coefficient_bound: int
    primitive_root: int


DEV = Parameters(DEV_PARAMETERS_ID, 97, 8, 1, 4, 2, 5)


def derive_polynomial(seed: bytes, row: int, column: int, params: Parameters) -> list[int]:
    limit = 65536 - (65536 % params.q)
    output: list[int] = []
    counter = 0
    while len(output) < params.n:
        h = hashlib.sha256()
        h.update(MATRIX_DOMAIN)
        h.update(params.identifier.to_bytes(4, "little"))
        h.update(seed)
        h.update(row.to_bytes(4, "little"))
        h.update(column.to_bytes(4, "little"))
        h.update(counter.to_bytes(8, "little"))
        block = h.digest()
        for offset in range(0, len(block), 2):
            candidate = int.from_bytes(block[offset : offset + 2], "little")
            if candidate < limit:
                output.append(candidate % params.q)
                if len(output) == params.n:
                    break
        counter += 1
    return output


def cyclic_ntt(values: list[int], root: int, invert: bool, q: int) -> list[int]:
    values = values.copy()
    n = len(values)
    j = 0
    for i in range(1, n):
        bit = n >> 1
        while j & bit:
            j ^= bit
            bit >>= 1
        j ^= bit
        if i < j:
            values[i], values[j] = values[j], values[i]

    selected_root = pow(root, q - 2, q) if invert else root
    length = 2
    while length <= n:
        step_root = pow(selected_root, n // length, q)
        for start in range(0, n, length):
            twiddle = 1
            for offset in range(length // 2):
                even = values[start + offset]
                odd = values[start + offset + length // 2] * twiddle % q
                values[start + offset] = (even + odd) % q
                values[start + offset + length // 2] = (even - odd) % q
                twiddle = twiddle * step_root % q
        length <<= 1

    if invert:
        inverse_n = pow(n, q - 2, q)
        values = [value * inverse_n % q for value in values]
    return values


def negacyclic_mul(a: list[int], b: list[int], params: Parameters) -> list[int]:
    psi = pow(params.primitive_root, (params.q - 1) // (2 * params.n), params.q)
    omega = psi * psi % params.q
    twist = 1
    left: list[int] = []
    right: list[int] = []
    for x, y in zip(a, b, strict=True):
        left.append(x * twist % params.q)
        right.append(y * twist % params.q)
        twist = twist * psi % params.q
    left = cyclic_ntt(left, omega, False, params.q)
    right = cyclic_ntt(right, omega, False, params.q)
    product = [x * y % params.q for x, y in zip(left, right, strict=True)]
    product = cyclic_ntt(product, omega, True, params.q)
    inverse_psi = pow(psi, params.q - 2, params.q)
    twist = 1
    output: list[int] = []
    for value in product:
        output.append(value * twist % params.q)
        twist = twist * inverse_psi % params.q
    return output


def negacyclic_mul_naive(a: list[int], b: list[int], q: int) -> list[int]:
    n = len(a)
    output = [0] * n
    for i, left in enumerate(a):
        for j, right in enumerate(b):
            degree = i + j
            if degree < n:
                output[degree] += left * right
            else:
                output[degree - n] -= left * right
    return [value % q for value in output]


def compute_target(seed: bytes, witness: list[int], params: Parameters) -> list[int]:
    assert len(witness) == params.columns * params.n
    assert all(abs(value) <= params.coefficient_bound for value in witness)
    output = [0] * (params.rows * params.n)
    for row in range(params.rows):
        for column in range(params.columns):
            matrix_poly = derive_polynomial(seed, row, column, params)
            witness_poly = [value % params.q for value in witness[column * params.n : (column + 1) * params.n]]
            product = negacyclic_mul(matrix_poly, witness_poly, params)
            for i, value in enumerate(product):
                output[row * params.n + i] = (output[row * params.n + i] + value) % params.q
    return output


def statement_digest(statement: dict) -> str:
    h = hashlib.sha256()
    h.update(STATEMENT_DOMAIN)
    h.update(int(statement["protocol_version"]).to_bytes(4, "little"))
    h.update(int(statement["parameter_id"]).to_bytes(4, "little"))
    h.update(bytes(statement["matrix_seed"]))
    h.update(bytes(statement["context"]))
    target = statement["target"]
    h.update(len(target).to_bytes(8, "little"))
    for coefficient in target:
        h.update(int(coefficient).to_bytes(4, "little"))
    return h.hexdigest()


def main() -> None:
    matrix_seed = hashlib.sha256(b"Ogunedo development matrix").digest()
    context = hashlib.sha256(b"Ogunedo development context").digest()
    witness = [((index * 7 + 3) % 5) - 2 for index in range(DEV.columns * DEV.n)]
    target = compute_target(matrix_seed, witness, DEV)

    # Independent NTT-vs-schoolbook test for every matrix/witness product.
    for column in range(DEV.columns):
        a = derive_polynomial(matrix_seed, 0, column, DEV)
        b = [value % DEV.q for value in witness[column * DEV.n : (column + 1) * DEV.n]]
        assert negacyclic_mul(a, b, DEV) == negacyclic_mul_naive(a, b, DEV.q)

    instance = {
        "statement": {
            "protocol_version": PROTOCOL_VERSION,
            "parameter_id": DEV.identifier,
            "matrix_seed": list(matrix_seed),
            "target": target,
            "context": list(context),
        },
        "witness": {"coeffs": witness},
    }
    fixture = ROOT / "fixtures" / "dev-instance.json"
    fixture.write_text(json.dumps(instance, indent=2) + "\n")
    public_fixture = ROOT / "fixtures" / "dev-statement.json"
    public_instance = {"statement": instance["statement"], "witness": None}
    public_fixture.write_text(json.dumps(public_instance, indent=2) + "\n")

    report = {
        "protocol_version": PROTOCOL_VERSION,
        "parameter_id_hex": f"0x{DEV.identifier:08x}",
        "matrix_seed_hex": matrix_seed.hex(),
        "context_hex": context.hex(),
        "target": target,
        "witness_l2_squared": sum(value * value for value in witness),
        "statement_digest_hex": statement_digest(instance["statement"]),
        "relation_digest_hex": hashlib.sha256(RELATION_DOMAIN).hexdigest(),
        "ntt_matches_naive": True,
        "private_fixture": fixture.relative_to(ROOT).as_posix(),
        "public_fixture": public_fixture.relative_to(ROOT).as_posix(),
    }
    (ROOT / "fixtures" / "reference-report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
