# Local proving policy

This policy protects the development laptop from resource exhaustion while working on Ogunedo.

## Hardware envelope

The primary local machine has:

- 12 GB physical RAM;
- a 500 GB drive;
- Windows with pagefile and possible WSL swap.

Do not disable the Windows pagefile or WSL swap.

## Command policy

Local compilation and tests must run sequentially. Heavy builds and proving commands must set:

```text
CARGO_BUILD_JOBS=2
```

Before every heavy SP1 command, inspect and record:

- free physical RAM;
- total/free virtual memory;
- pagefile allocation/current/peak usage;
- free disk space on the repository drive.

Never launch multiple proof jobs simultaneously.

## Required local sequence

Run in this order:

1. native/core checks;
2. guest build;
3. light SP1 execution;
4. cheaper development compressed proof;
5. production Groth16 only if the resource guard allows it.

Use:

```powershell
.\scripts\local_sp1_safe.ps1 -Mode inventory
.\scripts\local_sp1_safe.ps1 -Mode guest-build
.\scripts\local_sp1_safe.ps1 -Mode execute
.\scripts\local_sp1_safe.ps1 -Mode dev-proof
```

The script writes a resource report to:

```text
artifacts/local-resource-report.json
```

## Production Groth16 policy

Production Groth16 generation must not be forced on the 12 GB laptop. The local guard refuses production Groth16 when total physical RAM is below the configured envelope, currently 32 GiB.

Use one of these instead:

- the GitHub Actions SP1 workflow;
- a supported SP1 prover network;
- a larger dedicated proving machine.

An out-of-memory termination is a local resource failure, not a protocol failure.

## Artifact preservation

Before any heavy command:

- commit source changes;
- preserve successful intermediate artifacts;
- ensure proof output paths are not shared with another running command.

The local runner executes commands one at a time and records process peak working-set data where the OS exposes it. Because Cargo/SP1 may spawn child processes, the process-level peak is only a lower bound; the pagefile and system-memory snapshots are the authoritative local resource evidence.
