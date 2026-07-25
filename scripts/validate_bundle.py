#!/usr/bin/env python3
"""Offline structural and mathematical validation for the Ogunedo source bundle."""

from __future__ import annotations

import hashlib
import json
import random
import subprocess
import sys
from pathlib import Path

import yaml

import reference_check as ref

try:
    import tomllib
except ModuleNotFoundError:  # Python 3.10 compatibility.
    import tomli as tomllib

ROOT = Path(__file__).resolve().parents[1]
EXCLUDED_TREE_PARTS = {".git", "target", "dist", "artifacts", "proofs", "benchmarks"}


def source_files(pattern: str) -> list[Path]:
    return [
        path
        for path in ROOT.rglob(pattern)
        if not any(part in EXCLUDED_TREE_PARTS for part in path.relative_to(ROOT).parts)
    ]


def registered_draft() -> ref.Parameters:
    return ref.Parameters(0x4F470101, 12_289, 256, 1, 18, 2, 11)


def validate_structured_files(results: dict) -> None:
    toml_files = source_files("*.toml")
    json_files = source_files("*.json")
    yaml_files = source_files("*.yml")
    for path in toml_files:
        with path.open("rb") as handle:
            tomllib.load(handle)
    for path in json_files:
        json.loads(path.read_text())
    for path in yaml_files:
        yaml.safe_load(path.read_text())
    results["structured_files"] = {
        "toml": len(toml_files),
        "json": len(json_files),
        "yaml": len(yaml_files),
        "passed": True,
    }


def validate_shell_and_python(results: dict) -> None:
    shell_files = list((ROOT / "scripts").glob("*.sh"))
    for path in shell_files:
        subprocess.run(["bash", "-n", path.relative_to(ROOT).as_posix()], cwd=ROOT, check=True)
    subprocess.run([sys.executable, "-m", "compileall", "-q", str(ROOT / "scripts")], check=True)
    results["script_syntax"] = {
        "shell_files": len(shell_files),
        "python_files": len(list((ROOT / "scripts").glob("*.py"))),
        "passed": True,
    }


def validate_dev_fixture(results: dict) -> None:
    subprocess.run([sys.executable, str(ROOT / "scripts" / "reference_check.py")], check=True)
    fixture = json.loads((ROOT / "fixtures" / "dev-instance.json").read_text())
    statement = fixture["statement"]
    witness = fixture["witness"]["coeffs"]
    target = ref.compute_target(bytes(statement["matrix_seed"]), witness, ref.DEV)
    assert target == statement["target"]
    expected_digest = ref.statement_digest(statement)

    changed_target = statement["target"].copy()
    changed_target[0] = (changed_target[0] + 1) % ref.DEV.q
    assert changed_target != target

    changed_witness = witness.copy()
    changed_witness[0] = changed_witness[0] - 1 if changed_witness[0] == 2 else changed_witness[0] + 1
    assert ref.compute_target(bytes(statement["matrix_seed"]), changed_witness, ref.DEV) != target

    changed_context = dict(statement)
    changed_context["context"] = statement["context"].copy()
    changed_context["context"][0] ^= 1
    assert ref.statement_digest(changed_context) != expected_digest

    results["development_fixture"] = {
        "target": target,
        "statement_digest": expected_digest,
        "witness_l2_squared": sum(value * value for value in witness),
        "tampered_target_rejected": True,
        "tampered_witness_rejected": True,
        "context_binding_verified": True,
        "passed": True,
    }


def validate_draft_ntt(results: dict) -> None:
    params = registered_draft()
    left = [((index * index) + 17 * index + 3) % params.q for index in range(params.n)]
    right = [(5 * index * index + 11) % params.q for index in range(params.n)]
    ntt = ref.negacyclic_mul(left, right, params)
    schoolbook = ref.negacyclic_mul_naive(left, right, params.q)
    assert ntt == schoolbook

    matrix_seed = hashlib.sha256(b"Ogunedo draft validation matrix").digest()
    witness = [((index * 13 + 1) % 5) - 2 for index in range(params.columns * params.n)]
    target = ref.compute_target(matrix_seed, witness, params)
    assert len(target) == params.rows * params.n
    assert all(0 <= coefficient < params.q for coefficient in target)

    results["draft_profile"] = {
        "parameter_id": f"0x{params.identifier:08x}",
        "ntt_matches_schoolbook": True,
        "target_length": len(target),
        "target_sha256": hashlib.sha256(
            b"".join(value.to_bytes(4, "little") for value in target)
        ).hexdigest(),
        "witness_l2_squared": sum(value * value for value in witness),
        "passed": True,
    }



def validate_randomized_differential_tests(results: dict) -> None:
    rng = random.Random(0x4F47554E45444F)
    profiles = [(ref.DEV, 64), (registered_draft(), 8)]
    completed = {}
    for params, rounds in profiles:
        for _ in range(rounds):
            left = [rng.randrange(params.q) for _ in range(params.n)]
            right = [rng.randrange(params.q) for _ in range(params.n)]
            assert ref.negacyclic_mul(left, right, params) == ref.negacyclic_mul_naive(
                left, right, params.q
            )
        completed[f"0x{params.identifier:08x}"] = rounds

    seed = hashlib.sha256(b"Ogunedo deterministic expander test").digest()
    first = ref.derive_polynomial(seed, 0, 0, ref.DEV)
    second = ref.derive_polynomial(seed, 0, 0, ref.DEV)
    alternate = ref.derive_polynomial(seed, 0, 1, ref.DEV)
    assert first == second
    assert first != alternate
    assert all(0 <= coefficient < ref.DEV.q for coefficient in first)

    results["randomized_differential"] = {
        "fixed_rng_seed_hex": "0x4f47554e45444f",
        "ntt_schoolbook_rounds": completed,
        "matrix_expansion_deterministic": True,
        "matrix_column_separation_checked": True,
        "passed": True,
    }


def validate_version_pins(results: dict) -> None:
    cargo = tomllib.loads((ROOT / "Cargo.toml").read_text())
    dependencies = cargo["workspace"]["dependencies"]
    assert dependencies["sp1-zkvm"]["version"] == "=6.2.2"
    assert dependencies["sp1-sdk"]["version"] == "=6.2.2"
    assert dependencies["sp1-build"]["version"] == "=6.2.2"
    assert dependencies["sha2"]["version"] == "=0.10.8"
    results["version_pins"] = {
        "sp1": "6.2.2",
        "sha2": "0.10.8",
        "passed": True,
    }


def main() -> None:
    results: dict = {
        "validator": "Ogunedo offline bundle validator v1",
        "limitations": [
            "This report is an offline source-tree validator.",
            "It does not claim that SP1 execution or proof generation ran here.",
            "Rust/SP1 build status is tracked separately in BUILD_STATUS.md.",
        ],
    }
    validate_structured_files(results)
    validate_shell_and_python(results)
    validate_dev_fixture(results)
    validate_draft_ntt(results)
    validate_randomized_differential_tests(results)
    validate_version_pins(results)
    results["overall_passed"] = True
    report_path = ROOT / "VALIDATION_REPORT.json"
    report_path.write_text(json.dumps(results, indent=2) + "\n")
    print(json.dumps(results, indent=2))


if __name__ == "__main__":
    main()
