param(
    [string]$Version,
    [switch]$Help,
    [switch]$ConfirmTag,
    [switch]$SkipReleaseCheck
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$release = Get-Content (Join-Path $PSScriptRoot "..\release.json") -Raw | ConvertFrom-Json
$releaseVersion = if ($Version) { $Version } else { [string]$release.version }
$releaseTag = "v$releaseVersion"

if ($Help) {
    Write-Output "Usage: scripts\tag-release.ps1 [-Version <version>] -ConfirmTag [-SkipReleaseCheck]"
    Write-Output "Creates a local annotated tag only. It does not push or publish a release."
    Write-Output "Current release tag: $releaseTag"
    return
}

if (-not $ConfirmTag) {
    throw "Pass -ConfirmTag to create local release tag $releaseTag."
}

$status = git status --short
if ($status) {
    throw "Refusing to tag with a dirty worktree."
}

powershell -NoProfile -ExecutionPolicy Bypass -File scripts\assert-release-version.ps1 -Version $releaseVersion

if (-not $SkipReleaseCheck) {
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
}

$existing = git tag --list $releaseTag
if ($existing) {
    throw "Tag $releaseTag already exists."
}

git tag -a $releaseTag -m "Quotarelay $releaseVersion"
Write-Output "Created local tag $releaseTag."
Write-Output "Push it after owner approval: git push origin $releaseTag"
