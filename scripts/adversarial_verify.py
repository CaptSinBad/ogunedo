#!/usr/bin/env python3
"""Sequential adversarial verification driver for Ogunedo proof artifacts.

The script intentionally avoids network credentials and runs every verification
case in a fresh process. It is meant for release evidence on machines capable of
SP1 compressed verification.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import os
import shutil
import subprocess
import time
from pathlib import Path
from typing import Any


NETWORK_ENV_KEYS = (
    "NETWORK_PRIVATE_KEY",
    "NETWORK_RPC_URL",
    "SP1_PROVER",
)


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def run_command(command: list[str], timeout: int) -> dict[str, Any]:
    env = os.environ.copy()
    for key in NETWORK_ENV_KEYS:
        env.pop(key, None)
    started = time.time()
    timed = ["/usr/bin/time", "-v", *command]
    try:
        completed = subprocess.run(
            timed,
            cwd=Path.cwd(),
            env=env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=timeout,
            check=False,
        )
        elapsed = time.time() - started
        combined = completed.stdout + completed.stderr
        return {
            "exit_code": completed.returncode,
            "elapsed_seconds": round(elapsed, 3),
            "maximum_resident_set_kb": parse_peak_rss(combined),
            "stdout_tail": tail(completed.stdout),
            "stderr_tail": tail(completed.stderr),
        }
    except subprocess.TimeoutExpired as exc:
        return {
            "exit_code": None,
            "timed_out": True,
            "elapsed_seconds": round(time.time() - started, 3),
            "stdout_tail": tail(exc.stdout or ""),
            "stderr_tail": tail(exc.stderr or ""),
        }


def parse_peak_rss(output: str) -> int | None:
    marker = "Maximum resident set size (kbytes):"
    for line in output.splitlines():
        if marker in line:
            try:
                return int(line.split(marker, 1)[1].strip())
            except ValueError:
                return None
    return None


def tail(text: str, limit: int = 2000) -> str:
    if len(text) <= limit:
        return text
    return text[-limit:]


def mutate_statement(statement_path: Path, output_path: Path, mutation: str) -> None:
    data = load_json(statement_path)
    statement = data["statement"]
    if mutation == "context":
        statement["context"][0] ^= 1
    elif mutation == "target":
        statement["target"][0] = int(statement["target"][0]) + 1
    elif mutation == "matrix_seed":
        statement["matrix_seed"][0] ^= 1
    elif mutation == "protocol_version":
        statement["protocol_version"] = int(statement["protocol_version"]) + 1
    elif mutation == "parameter_id":
        statement["parameter_id"] = int(statement["parameter_id"]) + 1
    else:
        raise ValueError(f"unknown mutation {mutation}")
    write_json(output_path, data)


def validate_manifest(manifest: dict[str, Any], expected: dict[str, Any]) -> None:
    checks = {
        "proof_sha256": expected["proof_sha256"],
        "proof_size_bytes": expected["proof_size_bytes"],
        "statement_digest": expected["statement_digest"],
        "guest_elf_sha256": expected["guest_elf_sha256"],
        "vkey": expected["vkey"],
        "proof_mode": expected["proof_mode"],
        "network": expected["network"],
        "request_id": expected["request_id"],
    }
    for key, expected_value in checks.items():
        if manifest.get(key) != expected_value:
            raise ValueError(f"manifest field {key!r} mismatch")


def manifest_case(name: str, manifest: dict[str, Any], expected: dict[str, Any]) -> dict[str, Any]:
    started = time.time()
    try:
        validate_manifest(manifest, expected)
        exit_code = 0
        error = None
    except Exception as exc:  # noqa: BLE001 - report all validation failures.
        exit_code = 1
        error = str(exc)
    return {
        "name": name,
        "layer": "manifest",
        "expected_success": False,
        "passed": exit_code != 0,
        "exit_code": exit_code,
        "elapsed_seconds": round(time.time() - started, 3),
        "error": error,
    }


def opposite_mode(mode: str) -> str:
    if mode == "compressed":
        return "groth16"
    if mode == "groth16":
        return "compressed"
    return "compressed"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--proof", required=True, type=Path)
    parser.add_argument("--statement", required=True, type=Path)
    parser.add_argument("--preflight", required=True, type=Path)
    parser.add_argument("--receipt", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--tmpdir", required=True, type=Path)
    parser.add_argument("--expected-vkey", required=True)
    parser.add_argument("--timeout", type=int, default=900)
    args = parser.parse_args()

    args.tmpdir.mkdir(parents=True, exist_ok=True)
    shutil.rmtree(args.tmpdir)
    args.tmpdir.mkdir(parents=True)

    preflight = load_json(args.preflight)
    receipt = load_json(args.receipt)
    proof_sha = sha256_file(args.proof)
    proof_size = args.proof.stat().st_size
    statement_sha = sha256_file(args.statement)
    expected = {
        "proof_sha256": proof_sha,
        "proof_size_bytes": proof_size,
        "statement_digest": preflight["statement_digest"],
        "guest_elf_sha256": preflight["guest_elf_sha256"],
        "vkey": preflight["vkey"],
        "proof_mode": preflight["proof_mode"],
        "network": preflight["network"],
        "request_id": receipt["request_id"],
    }

    cases: list[dict[str, Any]] = []

    def add_verify_case(
        name: str,
        proof: Path,
        statement: Path,
        extra: list[str] | None = None,
    ) -> None:
        command = [
            str(args.binary),
            "verify",
            "--proof",
            str(proof),
            "--statement",
            str(statement),
            "--expected-proof-sha256",
            proof_sha,
            "--expected-proof-size-bytes",
            str(proof_size),
            "--expected-statement-sha256",
            statement_sha,
            *(extra or []),
        ]
        result = run_command(command, args.timeout)
        result.update(
            {
                "name": name,
                "layer": "proof-verifier",
                "expected_success": False,
                "passed": result.get("exit_code") not in (0, None),
                "command": command,
            }
        )
        cases.append(result)

    mutated_proof = args.tmpdir / "proof-mutated.bin"
    proof_bytes = bytearray(args.proof.read_bytes())
    proof_bytes[min(1024, len(proof_bytes) - 1)] ^= 1
    mutated_proof.write_bytes(proof_bytes)
    add_verify_case("mutated_proof_byte", mutated_proof, args.statement)

    truncated_proof = args.tmpdir / "proof-truncated.bin"
    truncated_proof.write_bytes(args.proof.read_bytes()[:-1])
    add_verify_case("truncated_proof", truncated_proof, args.statement)

    trailing_proof = args.tmpdir / "proof-trailing.bin"
    trailing_proof.write_bytes(args.proof.read_bytes() + b"OGUNEDO-TRAILING-BYTES")
    add_verify_case("proof_with_trailing_bytes", trailing_proof, args.statement)

    empty_proof = args.tmpdir / "proof-empty.bin"
    empty_proof.write_bytes(b"")
    add_verify_case("empty_proof", empty_proof, args.statement)

    symlink_proof = args.tmpdir / "proof-symlink.bin"
    try:
        symlink_proof.symlink_to(args.proof.resolve())
        add_verify_case("proof_symlink_substitution", symlink_proof, args.statement)
    except (OSError, NotImplementedError) as exc:
        cases.append(
            {
                "name": "proof_symlink_substitution",
                "layer": "proof-verifier",
                "expected_success": False,
                "passed": False,
                "exit_code": None,
                "skipped": True,
                "error": f"could not create symlink: {exc}",
            }
        )

    oversized_proof = args.tmpdir / "proof-oversized.bin"
    with oversized_proof.open("wb") as f:
        f.truncate(1024 * 1024 * 1024 + 1)
    add_verify_case("oversized_sparse_proof", oversized_proof, args.statement)

    for mutation in ("context", "target", "matrix_seed", "protocol_version", "parameter_id"):
        mutated_statement = args.tmpdir / f"statement-{mutation}.json"
        mutate_statement(args.statement, mutated_statement, mutation)
        add_verify_case(f"statement_{mutation}_tamper", args.proof, mutated_statement)

    mode_mismatch = opposite_mode(preflight["proof_mode"])
    add_verify_case(
        f"{preflight['proof_mode']}_proof_presented_as_{mode_mismatch}",
        args.proof,
        args.statement,
        ["--expected-mode", mode_mismatch],
    )

    add_verify_case(
        "wrong_expected_vkey",
        args.proof,
        args.statement,
        [
            "--expected-mode",
            preflight["proof_mode"],
            "--expected-vkey",
            "0xdeadbeef",
        ],
    )

    good_manifest = copy.deepcopy(expected)
    manifest_mutations = {
        "manifest_wrong_proof_hash": ("proof_sha256", "00" * 32),
        "manifest_wrong_statement_digest": ("statement_digest", "0x" + "11" * 32),
        "manifest_wrong_mode": ("proof_mode", mode_mismatch),
        "manifest_wrong_vkey": ("vkey", "0x" + "22" * 32),
        "manifest_wrong_network": ("network", "testnet"),
    }
    for name, (field, value) in manifest_mutations.items():
        bad_manifest = copy.deepcopy(good_manifest)
        bad_manifest[field] = value
        cases.append(manifest_case(name, bad_manifest, expected))

    passed = all(case["passed"] for case in cases)
    report = {
        "schema": "ogunedo-adversarial-verification-report-v1",
        "passed": passed,
        "proof_sha256": proof_sha,
        "proof_size_bytes": proof_size,
        "statement_sha256": statement_sha,
        "expected": expected,
        "network_credentials_present_in_driver_env": {
            key: key in os.environ for key in NETWORK_ENV_KEYS
        },
        "case_count": len(cases),
        "cases": cases,
    }
    write_json(args.output, report)
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
