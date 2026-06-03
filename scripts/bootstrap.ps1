param(
    [switch]$SkipFrontend
)

$ErrorActionPreference = "Stop"

function Invoke-BootstrapStep {
    param(
        [string]$Name,
        [scriptblock]$Command
    )

    Write-Output ""
    Write-Output "==> $Name"
    & $Command
    Write-Output "PASS: $Name"
}

Invoke-BootstrapStep "Rust toolchain" {
    rustc --version
    cargo --version
}

Invoke-BootstrapStep "Backend build" {
    cargo build -p mcp-server
}

if (-not $SkipFrontend) {
    Invoke-BootstrapStep "Node toolchain" {
        npm --version
    }

    Invoke-BootstrapStep "Control-plane build" {
        npm --prefix web/controlplane run build
    }
}

Write-Output ""
Write-Output "Next commands:"
Write-Output "  cargo run -p mcp-server"
Write-Output "  cargo run -p mcp-server -- --cli truth"
Write-Output "  powershell -NoProfile -ExecutionPolicy Bypass -File scripts\clean-check.ps1"
Write-Output ""
Write-Output "bootstrap passed"
