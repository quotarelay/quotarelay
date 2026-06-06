param(
    [switch]$SkipInstall
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$genericPresetPath = Join-Path $repoRoot "examples\mcp-client-presets\generic-stdio.json"
$installedPresetPath = Join-Path $repoRoot "examples\mcp-client-presets\installed-stdio.json"
$installRoot = Join-Path $repoRoot "target\install-smoke"
$installBin = Join-Path $installRoot "bin"

function Read-Preset {
    param([string]$Path)

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "missing MCP preset: $Path"
    }

    Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json
}

function Assert-Preset {
    param(
        [object]$Preset,
        [string]$ExpectedCommand,
        [string[]]$ExpectedArgs
    )

    if ($Preset.command -ne $ExpectedCommand) {
        throw "expected MCP preset command '$ExpectedCommand', got '$($Preset.command)'"
    }

    $actualArgs = @($Preset.args)
    if ($actualArgs.Count -ne $ExpectedArgs.Count) {
        throw "expected $($ExpectedArgs.Count) args for $ExpectedCommand, got $($actualArgs.Count)"
    }

    for ($index = 0; $index -lt $ExpectedArgs.Count; $index++) {
        if ($actualArgs[$index] -ne $ExpectedArgs[$index]) {
            throw "arg $index for $ExpectedCommand should be '$($ExpectedArgs[$index])', got '$($actualArgs[$index])'"
        }
    }
}

function Invoke-McpInitialize {
    param(
        [string]$Name,
        [string]$Command,
        [object[]]$PresetArgs
    )

    Write-Output ""
    Write-Output "==> $Name"

    $request = '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'
    $frame = "Content-Length: $($request.Length)`r`n`r`n$request"

    $startInfo = [System.Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $Command
    $startInfo.Arguments = (@($PresetArgs) -join " ")
    $startInfo.UseShellExecute = $false
    $startInfo.RedirectStandardInput = $true
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    $startInfo.CreateNoWindow = $true

    $process = [System.Diagnostics.Process]::Start($startInfo)
    $process.StandardInput.Write($frame)
    $process.StandardInput.Close()

    if (-not $process.WaitForExit(30000)) {
        $process.Kill()
        throw "$Name did not exit after stdio initialize"
    }

    $stdout = $process.StandardOutput.ReadToEnd()
    $stderr = $process.StandardError.ReadToEnd()

    if ($process.ExitCode -ne 0) {
        throw "$Name exited with $($process.ExitCode): $stderr"
    }

    $normalized = $stdout.Replace("`r`n", "`n")
    $parts = $normalized.Split(@("`n`n"), 2, [System.StringSplitOptions]::None)
    if ($parts.Count -ne 2) {
        throw "$Name did not emit a framed MCP response"
    }

    $response = $parts[1] | ConvertFrom-Json
    if ($response.result.serverInfo.name -ne "quotarelay") {
        throw "$Name returned unexpected server name: $($response.result.serverInfo.name)"
    }

    Write-Output "PASS: $Name"
}

$genericPreset = Read-Preset -Path $genericPresetPath
$installedPreset = Read-Preset -Path $installedPresetPath

Assert-Preset -Preset $genericPreset -ExpectedCommand "cargo" -ExpectedArgs @("run", "-p", "mcp-server")
Assert-Preset -Preset $installedPreset -ExpectedCommand "quotarelay-mcp" -ExpectedArgs @()

Invoke-McpInitialize -Name "source preset initialize" -Command $genericPreset.command -PresetArgs $genericPreset.args

if (-not $SkipInstall) {
    powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "install-smoke.ps1")
}

$oldPath = $env:PATH
try {
    $env:PATH = "$installBin$([System.IO.Path]::PathSeparator)$env:PATH"
    Invoke-McpInitialize -Name "installed preset initialize" -Command $installedPreset.command -PresetArgs $installedPreset.args
}
finally {
    $env:PATH = $oldPath
}

[PSCustomObject]@{
    ok = $true
    presets = @(
        "examples/mcp-client-presets/generic-stdio.json",
        "examples/mcp-client-presets/installed-stdio.json"
    )
} | ConvertTo-Json -Depth 3
