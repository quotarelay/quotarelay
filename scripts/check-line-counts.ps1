param(
    [int]$MaxLines = 500
)

$ErrorActionPreference = "Stop"

$sourceExtensions = @(
    ".cjs",
    ".css",
    ".html",
    ".js",
    ".jsx",
    ".mjs",
    ".ps1",
    ".py",
    ".rs",
    ".sh",
    ".ts",
    ".tsx"
)

$excludedDirectories = @(
    ".git",
    ".quotarelay",
    "coverage",
    "dist",
    "build",
    "node_modules",
    "target"
)

$generatedFiles = @(
    "Cargo.lock",
    "package-lock.json",
    "pnpm-lock.yaml",
    "yarn.lock"
)

function Test-IsExcludedPath {
    param([string]$Path)

    $parts = $Path -split '[\\/]'
    foreach ($part in $parts) {
        if ($excludedDirectories -contains $part) {
            return $true
        }
    }

    return $false
}

$root = (Get-Location).Path
$violations = @()
$checkedCount = 0

Get-ChildItem -Path $root -Recurse -File |
    Where-Object { $sourceExtensions -contains $_.Extension } |
    Where-Object { $generatedFiles -notcontains $_.Name } |
    Where-Object { -not (Test-IsExcludedPath $_.FullName.Substring($root.Length)) } |
    ForEach-Object {
        $lineCount = (Get-Content -LiteralPath $_.FullName).Count
        $checkedCount += 1

        if ($lineCount -gt $MaxLines) {
            $relativePath = $_.FullName.Substring($root.Length).TrimStart('\', '/')
            $violations += [pscustomobject]@{
                Path = $relativePath
                Lines = $lineCount
            }
        }
    }

if ($violations.Count -gt 0) {
    Write-Output "line-count guardrail failed (max $MaxLines lines)."
    $violations |
        Sort-Object -Property Lines -Descending |
        ForEach-Object { Write-Output "$($_.Lines) $($_.Path)" }
    exit 1
}

Write-Output "line-count guardrail passed (max $MaxLines lines). Checked $checkedCount source files."
