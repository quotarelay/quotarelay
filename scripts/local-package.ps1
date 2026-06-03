param(
    [switch]$SkipFrontend
)

$ErrorActionPreference = "Stop"

$Version = "0.1.0"
$PackageName = "quotarelay-$Version-local"
$RepoRoot = (Get-Location).Path
$PackageRoot = Join-Path $RepoRoot "target\local-package"
$StageRoot = Join-Path $PackageRoot $PackageName
$ArchivePath = Join-Path $PackageRoot "$PackageName.zip"
$BinarySource = Join-Path $RepoRoot "target\debug\mcp-server.exe"
$BinaryTargetDir = Join-Path $StageRoot "bin"
$BinaryTarget = Join-Path $BinaryTargetDir "mcp-server.exe"

function Invoke-PackageStep {
    param(
        [string]$Name,
        [scriptblock]$Command
    )

    Write-Output ""
    Write-Output "==> $Name"
    & $Command
    Write-Output "PASS: $Name"
}

function Assert-UnderPath {
    param(
        [string]$Path,
        [string]$ExpectedParent
    )

    $fullPath = [System.IO.Path]::GetFullPath($Path)
    $fullParent = [System.IO.Path]::GetFullPath($ExpectedParent)
    if (-not $fullParent.EndsWith([System.IO.Path]::DirectorySeparatorChar)) {
        $fullParent = "$fullParent$([System.IO.Path]::DirectorySeparatorChar)"
    }

    if (-not $fullPath.StartsWith($fullParent, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to modify path outside package root: $fullPath"
    }
}

Invoke-PackageStep "Backend build" {
    cargo build -p mcp-server
}

if (-not $SkipFrontend) {
    Invoke-PackageStep "Control-plane build" {
        npm --prefix web/controlplane run build
    }
}

Invoke-PackageStep "Prepare package directory" {
    Assert-UnderPath -Path $StageRoot -ExpectedParent $PackageRoot
    if (Test-Path $StageRoot) {
        Remove-Item -LiteralPath $StageRoot -Recurse -Force
    }
    New-Item -ItemType Directory -Path $BinaryTargetDir -Force | Out-Null
    New-Item -ItemType Directory -Path (Join-Path $StageRoot "docs") -Force | Out-Null
    New-Item -ItemType Directory -Path (Join-Path $StageRoot "scripts") -Force | Out-Null
}

Invoke-PackageStep "Copy local artifacts" {
    Copy-Item -LiteralPath $BinarySource -Destination $BinaryTarget
    Copy-Item -LiteralPath "README.md" -Destination (Join-Path $StageRoot "README.md")
    Copy-Item -LiteralPath "CHANGELOG.md" -Destination (Join-Path $StageRoot "CHANGELOG.md")
    Copy-Item -LiteralPath "docs\CONTROL_PLANE_LOCAL.md" -Destination (Join-Path $StageRoot "docs\CONTROL_PLANE_LOCAL.md")
    Copy-Item -LiteralPath "docs\MCP_CLIENT_CONFIG.md" -Destination (Join-Path $StageRoot "docs\MCP_CLIENT_CONFIG.md")
    Copy-Item -LiteralPath "docs\KNOWN_LIMITATIONS.md" -Destination (Join-Path $StageRoot "docs\KNOWN_LIMITATIONS.md")
    Copy-Item -LiteralPath "scripts\demo-local.ps1" -Destination (Join-Path $StageRoot "scripts\demo-local.ps1")
    if (-not $SkipFrontend) {
        Copy-Item -LiteralPath "web\controlplane\dist" -Destination (Join-Path $StageRoot "controlplane-dist") -Recurse
    }
}

Invoke-PackageStep "Write local run notes" {
    @(
        "# Quotarelay $Version Local Package",
        "",
        "This package is a local smoke artifact only. It is not published, signed, installed globally, or deployed.",
        "",
        "Run backend truth from this extracted package:",
        "",
        "````powershell",
        ".\bin\mcp-server.exe --cli truth",
        "````",
        "",
        "Run the local HTTP truth surface:",
        "",
        "````powershell",
        ".\bin\mcp-server.exe --http 127.0.0.1:3030",
        "````",
        "",
        "The control-plane static build is copied to `controlplane-dist` when the package script runs without `-SkipFrontend`.",
        "",
        "Repository sync, context assembly, memory, cache, and registered repository state remain local and explicit."
    ) | Set-Content -LiteralPath (Join-Path $StageRoot "RUNNING_LOCAL.md") -Encoding UTF8
}

Invoke-PackageStep "Package backend truth smoke" {
    & $BinaryTarget --cli truth | Out-Null
}

Invoke-PackageStep "Create zip archive" {
    Assert-UnderPath -Path $ArchivePath -ExpectedParent $PackageRoot
    if (Test-Path $ArchivePath) {
        Remove-Item -LiteralPath $ArchivePath -Force
    }
    Compress-Archive -LiteralPath $StageRoot -DestinationPath $ArchivePath
}

Write-Output ""
Write-Output "local package created: $ArchivePath"
