param(
    [ValidateSet("inventory", "guest-build", "execute", "dev-proof", "production-groth16")]
    [string] $Mode = "inventory",

    [string] $Instance = "fixtures/dev-instance.json",
    [string] $DevelopmentProof = "proofs/dev-compressed.bin",
    [string] $ProductionProof = "proofs/production-groth16.bin",
    [string] $Report = "artifacts/local-resource-report.json",

    [double] $MinimumFreeRamGiB = 3.0,
    [double] $MinimumFreeDiskGiB = 20.0,
    [double] $MinimumGroth16TotalRamGiB = 32.0
)

$ErrorActionPreference = "Stop"
$env:CARGO_BUILD_JOBS = "2"

function Convert-KiBToGiB([double] $value) {
    return [Math]::Round($value / 1024.0 / 1024.0, 3)
}

function Convert-BytesToGiB([double] $value) {
    return [Math]::Round($value / 1024.0 / 1024.0 / 1024.0, 3)
}

function Get-ResourceSnapshot([string] $label) {
    $os = Get-CimInstance Win32_OperatingSystem
    $pagefiles = @(Get-CimInstance Win32_PageFileUsage -ErrorAction SilentlyContinue)
    $root = (Get-Location).Path.Substring(0, 1)
    $drive = Get-PSDrive -Name $root

    $pagefileAllocatedMiB = 0
    $pagefileCurrentMiB = 0
    $pagefilePeakMiB = 0
    foreach ($pagefile in $pagefiles) {
        $pagefileAllocatedMiB += [int64] $pagefile.AllocatedBaseSize
        $pagefileCurrentMiB += [int64] $pagefile.CurrentUsage
        $pagefilePeakMiB += [int64] $pagefile.PeakUsage
    }

    [ordered]@{
        label = $label
        timestamp_utc = (Get-Date).ToUniversalTime().ToString("o")
        total_physical_ram_gib = Convert-KiBToGiB $os.TotalVisibleMemorySize
        free_physical_ram_gib = Convert-KiBToGiB $os.FreePhysicalMemory
        total_virtual_memory_gib = Convert-KiBToGiB $os.TotalVirtualMemorySize
        free_virtual_memory_gib = Convert-KiBToGiB $os.FreeVirtualMemory
        pagefile_allocated_gib = [Math]::Round($pagefileAllocatedMiB / 1024.0, 3)
        pagefile_current_usage_gib = [Math]::Round($pagefileCurrentMiB / 1024.0, 3)
        pagefile_peak_usage_gib = [Math]::Round($pagefilePeakMiB / 1024.0, 3)
        drive = "$($drive.Name):"
        drive_free_gib = Convert-BytesToGiB $drive.Free
        drive_used_gib = Convert-BytesToGiB $drive.Used
        cargo_build_jobs = $env:CARGO_BUILD_JOBS
    }
}

function Assert-ResourceEnvelope([string] $label, [bool] $heavy) {
    $snapshot = Get-ResourceSnapshot $label
    if ($heavy) {
        if ($snapshot.free_physical_ram_gib -lt $MinimumFreeRamGiB) {
            throw "Refusing heavy local SP1 command: free RAM $($snapshot.free_physical_ram_gib) GiB is below $MinimumFreeRamGiB GiB."
        }
        if ($snapshot.drive_free_gib -lt $MinimumFreeDiskGiB) {
            throw "Refusing heavy local SP1 command: free disk $($snapshot.drive_free_gib) GiB is below $MinimumFreeDiskGiB GiB."
        }
    }
    return $snapshot
}

function Invoke-LoggedCommand([string] $name, [string[]] $argv, [bool] $heavy) {
    $before = Assert-ResourceEnvelope "before:$name" $heavy
    $started = Get-Date
    Write-Host "==> $name"
    Write-Host "    $($argv -join ' ')"

    & $argv[0] @($argv[1..($argv.Length - 1)])
    $exitCode = if ($null -eq $LASTEXITCODE) { 0 } else { [int] $LASTEXITCODE }
    $ended = Get-Date
    $after = Get-ResourceSnapshot "after:$name"

    [ordered]@{
        name = $name
        command = $argv
        started_utc = $started.ToUniversalTime().ToString("o")
        ended_utc = $ended.ToUniversalTime().ToString("o")
        duration_seconds = [Math]::Round(($ended - $started).TotalSeconds, 3)
        exit_code = $exitCode
        peak_note = "PowerShell direct invocation is used for reliable exit-code propagation. Use system RAM/pagefile snapshots for machine-safety decisions; per-process child aggregate peak is not available from this runner."
        before = $before
        after = $after
    }
}

function Save-Report($steps, $status, $message) {
    $parent = Split-Path -Parent $Report
    if ($parent) {
        New-Item -ItemType Directory -Force -Path $parent | Out-Null
    }
    $payload = [ordered]@{
        status = $status
        message = $message
        generated_utc = (Get-Date).ToUniversalTime().ToString("o")
        hardware_policy = [ordered]@{
            cargo_build_jobs = 2
            sequential_commands = $true
            local_production_groth16_minimum_total_ram_gib = $MinimumGroth16TotalRamGiB
            minimum_free_ram_gib = $MinimumFreeRamGiB
            minimum_free_disk_gib = $MinimumFreeDiskGiB
            never_disable_windows_pagefile_or_wsl_swap = $true
        }
        snapshots_or_steps = $steps
    }
    $payload | ConvertTo-Json -Depth 8 | Set-Content -Encoding UTF8 -Path $Report
}

$steps = New-Object System.Collections.Generic.List[object]

try {
    if ($Mode -eq "inventory") {
        $steps.Add((Get-ResourceSnapshot "inventory"))
        Save-Report $steps "completed" "Resource inventory captured; no heavy SP1 command launched."
        Write-Host "Resource inventory written to $Report"
        exit 0
    }

    $steps.Add((Invoke-LoggedCommand "guest-build" @("cargo", "check", "-p", "ogunedo-program", "--locked") $false))
    if ($steps[-1].exit_code -ne 0) {
        throw "guest-build failed with exit code $($steps[-1].exit_code)"
    }

    if ($Mode -in @("execute", "dev-proof", "production-groth16")) {
        $steps.Add((Invoke-LoggedCommand "light-execution" @("cargo", "run", "--release", "-p", "ogunedo-cli", "--", "execute", "--instance", $Instance) $true))
        if ($steps[-1].exit_code -ne 0) {
            throw "light-execution failed with exit code $($steps[-1].exit_code)"
        }
    }

    if ($Mode -in @("dev-proof", "production-groth16")) {
        $steps.Add((Invoke-LoggedCommand "development-compressed-proof" @("cargo", "run", "--release", "-p", "ogunedo-cli", "--", "prove", "--instance", $Instance, "--mode", "compressed", "--output", $DevelopmentProof, "--allow-unreviewed-parameters") $true))
        if ($steps[-1].exit_code -ne 0) {
            throw "development proof failed with exit code $($steps[-1].exit_code)"
        }
    }

    if ($Mode -eq "production-groth16") {
        $snapshot = Assert-ResourceEnvelope "before:production-groth16-policy" $true
        if ($snapshot.total_physical_ram_gib -lt $MinimumGroth16TotalRamGiB) {
            throw "Refusing local production Groth16: total RAM $($snapshot.total_physical_ram_gib) GiB is below $MinimumGroth16TotalRamGiB GiB. Use GitHub Actions or a supported SP1 prover network."
        }
        $steps.Add((Invoke-LoggedCommand "production-groth16-proof" @("cargo", "run", "--release", "-p", "ogunedo-cli", "--", "prove", "--instance", $Instance, "--mode", "groth16", "--output", $ProductionProof, "--allow-unreviewed-parameters") $true))
        if ($steps[-1].exit_code -ne 0) {
            throw "production Groth16 failed with exit code $($steps[-1].exit_code)"
        }
    }

    Save-Report $steps "completed" "Requested local SP1-safe mode completed."
} catch {
    Save-Report $steps "stopped" $_.Exception.Message
    Write-Error $_.Exception.Message
    exit 1
}
