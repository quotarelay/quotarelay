$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$installRoot = Join-Path $repoRoot "target\install-smoke"
$binName = if ([System.IO.Path]::DirectorySeparatorChar -eq '\') { "quotarelay-mcp.exe" } else { "quotarelay-mcp" }
$installedBin = Join-Path $installRoot "bin\$binName"

function Invoke-InstallSmokeStep {
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

Invoke-InstallSmokeStep "cargo install quotarelay-mcp" {
    cargo install --path apps/mcp-server --root $installRoot --debug --force --locked --offline
}

Invoke-InstallSmokeStep "installed command truth" {
    & $installedBin --cli truth | Out-Null
}

[PSCustomObject]@{
    ok = $true
    command = "quotarelay-mcp"
    install_root = $installRoot
    installed_bin = $installedBin
} | ConvertTo-Json -Depth 3
