$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$privateUser = "bro" + "grammer"
$windowsPath = "C:\\Users\\" + $privateUser
$posixPath = "C:/Users/" + $privateUser
$escapedWindowsPath = [regex]::Escape($windowsPath)
$escapedPosixPath = [regex]::Escape($posixPath)
$pattern = "(UNLICENSED|$escapedWindowsPath|$escapedPosixPath|BROGRA|planning-session-context|quotarelay-npm-placeholder|api[_-]?key|PRIVATE KEY|BEGIN RSA|BEGIN OPENSSH|Authorization:|Bearer |sk-[A-Za-z0-9]{20,})"
$excludes = @(
    "!target/**",
    "!web/controlplane/node_modules/**",
    "!web/controlplane/dist/**",
    "!scripts/public-surface-scan.ps1"
)

function Test-ExcludedPath {
    param(
        [string]$Path
    )

    $normalized = $Path.Replace("\", "/")
    return $normalized.StartsWith("target/") -or
        $normalized.StartsWith("web/controlplane/node_modules/") -or
        $normalized.StartsWith("web/controlplane/dist/") -or
        $normalized -eq "scripts/public-surface-scan.ps1"
}

if (Get-Command rg -ErrorAction SilentlyContinue) {
    $args = @("-n", $pattern, "-S", ".")
    foreach ($exclude in $excludes) {
        $args += "-g"
        $args += $exclude
    }

    $output = & rg @args
    if ($LASTEXITCODE -eq 0) {
        $output
        throw "public-surface scan found blocked content"
    }
    if ($LASTEXITCODE -gt 1) {
        throw "public-surface scan failed with exit code $LASTEXITCODE"
    }
} else {
    $trackedFiles = git ls-files
    if ($LASTEXITCODE -ne 0) {
        throw "public-surface scan could not list tracked files"
    }

    $matches = @()
    foreach ($file in $trackedFiles) {
        if (Test-ExcludedPath -Path $file) {
            continue
        }
        if (-not (Test-Path -LiteralPath $file -PathType Leaf)) {
            continue
        }

        $fileMatches = Select-String -LiteralPath $file -Pattern $pattern -ErrorAction SilentlyContinue
        if ($fileMatches) {
            $matches += $fileMatches
        }
    }

    if ($matches.Count -gt 0) {
        $matches | ForEach-Object { "$($_.Path):$($_.LineNumber):$($_.Line)" }
        throw "public-surface scan found blocked content"
    }
}

[PSCustomObject]@{
    ok = $true
    scanned = "."
    excluded = $excludes
} | ConvertTo-Json -Depth 3
