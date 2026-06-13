param(
    [string]$Version,
    [switch]$SkipReleaseCheck,
    [switch]$Audit,
    [switch]$BuildLocalPackage
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$release = Get-Content (Join-Path $repoRoot "release.json") -Raw | ConvertFrom-Json
$releaseVersion = if ($Version) { $Version } else { [string]$release.version }
$releaseTag = "v$releaseVersion"

function Invoke-ReleasePrepStep {
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

Invoke-ReleasePrepStep "version manifest" {
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts\assert-release-version.ps1 -Version $releaseVersion
}

if (-not $SkipReleaseCheck) {
    Invoke-ReleasePrepStep "release check" {
        powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
    }
}

if ($Audit) {
    Invoke-ReleasePrepStep "frontend dependency audit" {
        npm.cmd --prefix web/controlplane audit --audit-level=moderate
    }
}

if ($BuildLocalPackage) {
    Invoke-ReleasePrepStep "local package" {
        powershell -NoProfile -ExecutionPolicy Bypass -File scripts\local-package.ps1
    }
}

Write-Output ""
Write-Output "Release prep passed for $releaseTag."
Write-Output "Create the local tag only after reviewing the staged release:"
Write-Output "  powershell -NoProfile -ExecutionPolicy Bypass -File scripts\tag-release.ps1 -Version $releaseVersion -ConfirmTag"
Write-Output "Push the tag only after owner approval:"
Write-Output "  git push origin $releaseTag"
