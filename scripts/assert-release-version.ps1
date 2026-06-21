param(
    [string]$Version
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$releasePath = Join-Path $repoRoot "release.json"
$release = Get-Content $releasePath -Raw | ConvertFrom-Json
$expectedVersion = if ($Version) { $Version } else { $release.version }
$expectedTag = "v$expectedVersion"

function Assert-Equal {
    param(
        [string]$Name,
        [string]$Actual,
        [string]$Expected
    )

    if ($Actual -ne $Expected) {
        throw "$Name expected '$Expected' but found '$Actual'"
    }
}

function Read-JsonVersion {
    param([string]$Path)
    $json = Get-Content (Join-Path $repoRoot $Path) -Raw | ConvertFrom-Json
    return [string]$json.version
}

$cargoText = Get-Content (Join-Path $repoRoot "Cargo.toml") -Raw
$cargoVersion = [regex]::Match(
    $cargoText,
    '(?ms)\[workspace\.package\].*?version\s*=\s*"([^"]+)"'
).Groups[1].Value

Assert-Equal "release.json version" ([string]$release.version) $expectedVersion
Assert-Equal "release.json tag" ([string]$release.tag) $expectedTag
Assert-Equal "root package.json version" (Read-JsonVersion "package.json") $expectedVersion
Assert-Equal "control-plane package.json version" (Read-JsonVersion "web\controlplane\package.json") $expectedVersion
Assert-Equal "Cargo workspace version" $cargoVersion $expectedVersion

$localPackage = Get-Content (Join-Path $repoRoot "scripts\local-package.ps1") -Raw
if (-not $localPackage.Contains('$Version = "' + $expectedVersion + '"')) {
    throw "scripts\local-package.ps1 does not use version $expectedVersion"
}

$readme = Get-Content (Join-Path $repoRoot "README.md") -Raw
if (-not $readme.Contains("Current local MVP version: ``$expectedVersion``.")) {
    throw "README.md does not report version $expectedVersion"
}

$changelog = Get-Content (Join-Path $repoRoot "CHANGELOG.md") -Raw
if (-not $changelog.Contains("## $expectedVersion -")) {
    throw "CHANGELOG.md is missing a $expectedVersion release heading"
}

$versioning = Get-Content (Join-Path $repoRoot "docs\VERSIONING.md") -Raw
if (-not $versioning.Contains("Workspace crates: ``$expectedVersion``")) {
    throw "docs\VERSIONING.md is missing workspace version $expectedVersion"
}
if (-not $versioning.Contains("Control plane package: ``$expectedVersion``")) {
    throw "docs\VERSIONING.md is missing control-plane version $expectedVersion"
}

[PSCustomObject]@{
    ok = $true
    version = $expectedVersion
    tag = $expectedTag
    draft = [bool]$release.draft
    prerelease = [bool]$release.prerelease
    local_only = [bool]$release.local_only
    publish = [bool]$release.publish
} | ConvertTo-Json -Depth 4
