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
$BrandedBinarySource = Join-Path $RepoRoot "target\debug\quotarelay-mcp.exe"
$BinaryTargetDir = Join-Path $StageRoot "bin"
$BinaryTarget = Join-Path $BinaryTargetDir "mcp-server.exe"
$BrandedBinaryTarget = Join-Path $BinaryTargetDir "quotarelay-mcp.exe"

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

function Assert-PackageFile {
    param(
        [string]$RelativePath
    )

    $packagePath = Join-Path $StageRoot $RelativePath
    if (-not (Test-Path -LiteralPath $packagePath -PathType Leaf)) {
        throw "Package is missing required file: $RelativePath"
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
    Copy-Item -LiteralPath $BrandedBinarySource -Destination $BrandedBinaryTarget
    Copy-Item -LiteralPath "README.md" -Destination (Join-Path $StageRoot "README.md")
    Copy-Item -LiteralPath "CHANGELOG.md" -Destination (Join-Path $StageRoot "CHANGELOG.md")
    Copy-Item -LiteralPath "LICENSE" -Destination (Join-Path $StageRoot "LICENSE")
    Copy-Item -LiteralPath "NOTICE" -Destination (Join-Path $StageRoot "NOTICE")
    Copy-Item -LiteralPath "docs\CONTROL_PLANE_LOCAL.md" -Destination (Join-Path $StageRoot "docs\CONTROL_PLANE_LOCAL.md")
    Copy-Item -LiteralPath "docs\DEPLOYMENT_READINESS.md" -Destination (Join-Path $StageRoot "docs\DEPLOYMENT_READINESS.md")
    Copy-Item -LiteralPath "docs\MCP_CLIENT_CONFIG.md" -Destination (Join-Path $StageRoot "docs\MCP_CLIENT_CONFIG.md")
    Copy-Item -LiteralPath "docs\KNOWN_LIMITATIONS.md" -Destination (Join-Path $StageRoot "docs\KNOWN_LIMITATIONS.md")
    Copy-Item -LiteralPath "docs\PLATFORM_SURFACES.md" -Destination (Join-Path $StageRoot "docs\PLATFORM_SURFACES.md")
    Copy-Item -LiteralPath "docs\PRIVATE_DEPLOYMENT_PLAN.md" -Destination (Join-Path $StageRoot "docs\PRIVATE_DEPLOYMENT_PLAN.md")
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
        ".\bin\quotarelay-mcp.exe --cli truth",
        "````",
        "",
        "Show the console usage snapshot:",
        "",
        "````powershell",
        ".\bin\quotarelay-mcp.exe --cli usage",
        "````",
        "",
        "Run the local HTTP truth surface:",
        "",
        "````powershell",
        ".\bin\quotarelay-mcp.exe --http 127.0.0.1:3030",
        "````",
        "",
        "Headless mode is the HTTP truth command above. The control-plane static build is copied to `controlplane-dist` when the package script runs without `-SkipFrontend`.",
        "",
        "Repository sync, context assembly, memory, cache, and registered repository state remain local and explicit."
    ) | Set-Content -LiteralPath (Join-Path $StageRoot "RUNNING_LOCAL.md") -Encoding UTF8
}

Invoke-PackageStep "Package backend truth smoke" {
    & $BrandedBinaryTarget --cli truth | Out-Null
}

Invoke-PackageStep "Package contents smoke" {
    Assert-PackageFile -RelativePath "README.md"
    Assert-PackageFile -RelativePath "bin\quotarelay-mcp.exe"
    Assert-PackageFile -RelativePath "CHANGELOG.md"
    Assert-PackageFile -RelativePath "LICENSE"
    Assert-PackageFile -RelativePath "NOTICE"
    Assert-PackageFile -RelativePath "RUNNING_LOCAL.md"
    Assert-PackageFile -RelativePath "docs\CONTROL_PLANE_LOCAL.md"
    Assert-PackageFile -RelativePath "docs\DEPLOYMENT_READINESS.md"
    Assert-PackageFile -RelativePath "docs\MCP_CLIENT_CONFIG.md"
    Assert-PackageFile -RelativePath "docs\KNOWN_LIMITATIONS.md"
    Assert-PackageFile -RelativePath "docs\PLATFORM_SURFACES.md"
    Assert-PackageFile -RelativePath "docs\PRIVATE_DEPLOYMENT_PLAN.md"
    Assert-PackageFile -RelativePath "scripts\demo-local.ps1"
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
