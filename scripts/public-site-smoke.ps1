param(
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$controlPlaneRoot = Join-Path $repoRoot "web\controlplane"
$distRoot = Join-Path $controlPlaneRoot "dist"
$manifestPath = Join-Path $distRoot ".vite\manifest.json"
$indexPath = Join-Path $distRoot "index.html"

if (-not $SkipBuild) {
    npm.cmd --prefix $controlPlaneRoot run build
    if ($LASTEXITCODE -ne 0) {
        throw "control-plane build failed with exit code $LASTEXITCODE"
    }
}

if (-not (Test-Path $manifestPath)) {
    throw "missing Vite manifest at $manifestPath"
}
if (-not (Test-Path $indexPath)) {
    throw "missing built index.html at $indexPath"
}

$manifest = Get-Content $manifestPath -Raw | ConvertFrom-Json
$manifestProperties = $manifest.PSObject.Properties.Name

foreach ($route in @("src/pages/index.tera", "src/pages/control-plane.tera")) {
    if ($manifestProperties -notcontains $route) {
        throw "manifest does not include $route"
    }
}

$indexHtml = Get-Content $indexPath -Raw
foreach ($needle in @(
        "<title>Quotarelay</title>",
        "Apache-2.0 local MCP context compression for coding agents"
    )) {
    if (-not $indexHtml.Contains($needle)) {
        throw "built index.html is missing expected public metadata: $needle"
    }
}

$assetDir = Join-Path $distRoot "assets"
$routeAssets = Get-ChildItem $assetDir -Filter "*.js" | Where-Object {
    (Get-Content $_.FullName -Raw).Contains("/control-plane") -or
    (Get-Content $_.FullName -Raw).Contains("Local MCP context dashboard")
}

if ($routeAssets.Count -eq 0) {
    throw "built assets do not include public/control-plane route metadata"
}

$assetText = ($routeAssets | ForEach-Object { Get-Content $_.FullName -Raw }) -join "`n"
foreach ($needle in @(
        "Local MCP dashboard",
        "Backend:",
        "Repo:",
        "Context:",
        "Memory:",
        "MCP tool usage",
        "Provider calls",
        "Recent runs",
        "Memory matches",
        "System health",
        "System",
        "Light",
        "Dark",
        "Backend offline",
        "Provider calls",
        "Cache",
        "Repository",
        "MCP tools",
        "quotarelay-mcp",
        "94.36%",
        "No repository selected",
        "Boundaries"
    )) {
    if (-not $assetText.Contains($needle)) {
        throw "built assets are missing expected agent-tool positioning: $needle"
    }
}

$publicText = "$indexHtml`n$assetText"
foreach ($blocked in @(
        "Start a paid trial",
        "paid plan required",
        "pricing tier",
        "requires hosted login",
        "exact provider billing savings",
        "guaranteed savings",
        "production hosted ready",
        "SOC 2 compliant"
    )) {
    if ($publicText.Contains($blocked)) {
        throw "built public site includes blocked overclaim or paid-gate language: $blocked"
    }
}

[PSCustomObject]@{
    ok = $true
    routes = @("/", "/control-plane")
    manifest = $manifestPath
    public_metadata = "Quotarelay"
    route_asset_count = $routeAssets.Count
} | ConvertTo-Json -Depth 4
