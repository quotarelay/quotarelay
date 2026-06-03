$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$privateUser = "bro" + "grammer"
$windowsPath = "C:\\Users\\" + $privateUser
$posixPath = "C:/Users/" + $privateUser
$pattern = "(UNLICENSED|$windowsPath|$posixPath|BROGRA|planning-session-context|quotarelay-npm-placeholder|api[_-]?key|PRIVATE KEY|BEGIN RSA|BEGIN OPENSSH|Authorization:|Bearer |sk-[A-Za-z0-9]{20,})"
$excludes = @(
    "!target/**",
    "!web/controlplane/node_modules/**",
    "!web/controlplane/dist/**",
    "!scripts/public-surface-scan.ps1"
)

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

[PSCustomObject]@{
    ok = $true
    scanned = "."
    excluded = $excludes
} | ConvertTo-Json -Depth 3
