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
    & $Command
    Write-Output "PASS: $Name"
}

$cleanArgs = @()
if ($SkipFrontend) {
    $cleanArgs += "-SkipFrontend"
}

Invoke-ReleaseStep "clean check" {
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts\clean-check.ps1 @cleanArgs
}

Invoke-ReleaseStep "local demo smoke" {
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts\demo-local.ps1
}

Invoke-ReleaseStep "docs sanity" {
    rg -n "local MVP|Deferred platform|provider billing|docs/TRUTH_MATRIX.md|docs/LOCAL_STATE_PRIVACY.md|docs/TROUBLESHOOTING.md" README.md docs
}

Write-Output ""
Write-Output "release-check passed"
