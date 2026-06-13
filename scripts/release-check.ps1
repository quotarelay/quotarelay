param(
    [switch]$SkipFrontend
)

$ErrorActionPreference = "Stop"

function Invoke-ReleaseStep {
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

$cleanArgs = @()
if ($SkipFrontend) {
    $cleanArgs += "-SkipFrontend"
}

Invoke-ReleaseStep "clean check" {
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts\clean-check.ps1 @cleanArgs
}

Invoke-ReleaseStep "release version manifest" {
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts\assert-release-version.ps1
}

Invoke-ReleaseStep "local demo smoke" {
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts\demo-local.ps1
}

Invoke-ReleaseStep "token-saver benchmark" {
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts\token-saver-benchmark.ps1
}

Invoke-ReleaseStep "local install smoke" {
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts\install-smoke.ps1
}

Invoke-ReleaseStep "MCP preset smoke" {
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts\mcp-preset-smoke.ps1 -SkipInstall
}

if (-not $SkipFrontend) {
    Invoke-ReleaseStep "public site smoke" {
        powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-site-smoke.ps1 -SkipBuild
    }
}

Invoke-ReleaseStep "public surface scan" {
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-surface-scan.ps1
}

Invoke-ReleaseStep "docs sanity" {
    rg -n "local MVP|Deferred platform|provider billing|docs/TRUTH_MATRIX.md|docs/LOCAL_STATE_PRIVACY.md|docs/TROUBLESHOOTING.md" README.md docs
}

Write-Output ""
Write-Output "release-check passed"
