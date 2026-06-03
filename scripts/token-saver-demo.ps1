param(
    [string]$WorkspaceRoot = ""
)

$ErrorActionPreference = "Stop"

$benchmark = Join-Path $PSScriptRoot "token-saver-benchmark.ps1"
& $benchmark -WorkspaceRoot $WorkspaceRoot
