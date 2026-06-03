param(
    [switch]$SkipFrontend
)

$ErrorActionPreference = "Stop"

function Invoke-CleanStep {
    param(
        [string]$Name,
        [scriptblock]$Command
    )

    Write-Output ""
    Write-Output "==> $Name"
    $global:LASTEXITCODE = 0
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "$Name failed with exit code $LASTEXITCODE"
    }
    Write-Output "PASS: $Name"
}

Invoke-CleanStep "cargo fmt --all --check" {
    cargo fmt --all --check
}

Invoke-CleanStep "source line-count guardrail" {
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-line-counts.ps1
}

Invoke-CleanStep "repo-index tests" {
    cargo test -p repo-index
}

Invoke-CleanStep "context-engine tests" {
    cargo test -p context-engine
}

Invoke-CleanStep "mcp-server tests" {
    cargo test -p mcp-server
}

if (-not $SkipFrontend) {
    Invoke-CleanStep "control-plane build" {
        npm --prefix web/controlplane run build
    }
}

Write-Output ""
Write-Output "clean-check passed"
