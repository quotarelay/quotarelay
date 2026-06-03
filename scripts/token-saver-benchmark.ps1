param(
    [string]$WorkspaceRoot = ""
)

$ErrorActionPreference = "Stop"

function New-BenchmarkRequest {
    param(
        [int]$Id,
        [string]$Name,
        [hashtable]$Arguments
    )

    return @{
        jsonrpc = "2.0"
        id = $Id
        method = "tools/call"
        params = @{
            name = $Name
            arguments = $Arguments
        }
    }
}

function ConvertTo-FramedJson {
    param([object[]]$Requests)

    $framed = ""
    foreach ($request in $Requests) {
        $json = $request | ConvertTo-Json -Depth 24 -Compress
        $length = [Text.Encoding]::UTF8.GetByteCount($json)
        $framed += "Content-Length: $length`r`n`r`n$json"
    }
    return $framed
}

function Invoke-McpBatch {
    param([object[]]$Requests)

    $server = Join-Path (Get-Location) "target\debug\mcp-server.exe"
    if (-not (Test-Path -LiteralPath $server)) {
        cargo build -p mcp-server | Out-Null
    }

    $raw = ConvertTo-FramedJson -Requests $Requests | & $server | Out-String
    $payloads = $raw -split 'Content-Length: \d+\r?\n\r?\n' |
        Where-Object { $_.Trim().StartsWith("{") }

    return $payloads | ForEach-Object { $_ | ConvertFrom-Json }
}

function Get-ToolPayload {
    param([object]$Response)

    $text = $Response.result.content[0].text
    return $text | ConvertFrom-Json
}

function Get-JsonByteCount {
    param([object]$Value)

    $json = $Value | ConvertTo-Json -Depth 32 -Compress
    return [Text.Encoding]::UTF8.GetByteCount($json)
}

function Get-ApproxTokens {
    param([int]$Bytes)
    return [Math]::Ceiling($Bytes / 4)
}

function New-BenchmarkMetric {
    param(
        [string]$Name,
        [int]$RawBytes,
        [object]$Payload
    )

    $payloadBytes = Get-JsonByteCount $Payload
    $reduction = if ($RawBytes -gt 0) {
        [Math]::Round((1 - ($payloadBytes / $RawBytes)) * 100, 2)
    } else {
        0
    }

    return [ordered]@{
        name = $Name
        raw_bytes = $RawBytes
        packet_bytes = $payloadBytes
        approximate_raw_tokens = Get-ApproxTokens $RawBytes
        approximate_packet_tokens = Get-ApproxTokens $payloadBytes
        estimated_reduction_percent = $reduction
    }
}

function Write-FixtureRepo {
    param([string]$RepoRoot)

    New-Item -ItemType Directory -Force -Path $RepoRoot | Out-Null
    New-Item -ItemType Directory -Force -Path (Join-Path $RepoRoot "src") | Out-Null
    New-Item -ItemType Directory -Force -Path (Join-Path $RepoRoot "docs") | Out-Null

    $padding = (1..120 | ForEach-Object { "padding line $_ for broad raw repository context" }) -join "`n"
    Set-Content -LiteralPath (Join-Path $RepoRoot "README.md") -Value @"
# Benchmark Fixture

This fixture has a needle workflow plus enough unrelated text to make broad raw context visibly larger.

$padding
"@

    Set-Content -LiteralPath (Join-Path $RepoRoot "src\lib.rs") -Value @"
pub mod workflow;

pub struct BenchmarkNeedle {
    pub label: String,
}

pub fn needle_entrypoint() -> &'static str {
    "needle workflow entrypoint"
}

$padding
"@

    Set-Content -LiteralPath (Join-Path $RepoRoot "src\workflow.rs") -Value @"
pub fn run_workflow() -> &'static str {
    "needle workflow should stay bounded"
}

pub fn unrelated_helper() -> &'static str {
    "other context"
}

$padding
"@

    Set-Content -LiteralPath (Join-Path $RepoRoot "docs\guide.md") -Value @"
# Operator Guide

The decision memory should say the needle workflow prefers bounded local context.

$padding
"@
}

function Get-RawRepoBytes {
    param([string]$RepoRoot)

    $total = 0
    Get-ChildItem -LiteralPath $RepoRoot -Recurse -File |
        Where-Object { $_.FullName -notmatch '\\.quotarelay\\' } |
        ForEach-Object {
            $total += [Text.Encoding]::UTF8.GetByteCount((Get-Content -LiteralPath $_.FullName -Raw))
        }
    return $total
}

if ([string]::IsNullOrWhiteSpace($WorkspaceRoot)) {
    $WorkspaceRoot = Join-Path ([IO.Path]::GetTempPath()) ("quotarelay-token-benchmark-" + [Guid]::NewGuid())
}

$repoRoot = Join-Path $WorkspaceRoot "repo"
New-Item -ItemType Directory -Force -Path $WorkspaceRoot | Out-Null
Write-FixtureRepo -RepoRoot $repoRoot
$rawBytesBeforeDiff = Get-RawRepoBytes -RepoRoot $repoRoot

$setupRequests = @(
    New-BenchmarkRequest 1 "sync_repo" @{ root = $repoRoot }
    New-BenchmarkRequest 2 "memory_write" @{
        root = $repoRoot
        title = "Needle decision"
        content = "For benchmark work, the needle workflow should use bounded local context."
        tags = @("needle", "benchmark")
        profile = "decision"
    }
    New-BenchmarkRequest 3 "cache_inspect" @{ root = $repoRoot }
    New-BenchmarkRequest 4 "assemble_context" @{
        root = $repoRoot
        mode = "exact_search"
        query = "needle"
        limit = 3
    }
    New-BenchmarkRequest 5 "cache_inspect" @{ root = $repoRoot }
    New-BenchmarkRequest 6 "assemble_context" @{
        root = $repoRoot
        mode = "exact_search"
        query = "needle"
        limit = 3
    }
    New-BenchmarkRequest 7 "cache_inspect" @{ root = $repoRoot }
    New-BenchmarkRequest 8 "assemble_context" @{
        root = $repoRoot
        mode = "overview"
        limit = 3
    }
    New-BenchmarkRequest 9 "handoff_packet" @{
        root = $repoRoot
        active_task = "Continue the benchmark needle workflow"
        mode = "exact_search"
        query = "needle"
        limit = 3
    }
)

$setupResponses = Invoke-McpBatch -Requests $setupRequests
$cacheBefore = Get-ToolPayload $setupResponses[2]
$exactFirst = Get-ToolPayload $setupResponses[3]
$cacheAfterFirst = Get-ToolPayload $setupResponses[4]
$exactSecond = Get-ToolPayload $setupResponses[5]
$cacheAfterSecond = Get-ToolPayload $setupResponses[6]
$overview = Get-ToolPayload $setupResponses[7]
$handoff = Get-ToolPayload $setupResponses[8]

if (($cacheBefore.exact_search_cache.item_count -ne 0) -or ($cacheAfterFirst.exact_search_cache.item_count -ne 1) -or ($cacheAfterSecond.exact_search_cache.item_count -ne 1)) {
    throw "exact_search cache count did not show the expected miss then stable cached entry"
}

if ((Get-JsonByteCount $exactFirst) -ne (Get-JsonByteCount $exactSecond)) {
    throw "repeated exact_search did not return the same cached packet size"
}

Add-Content -LiteralPath (Join-Path $repoRoot "src\workflow.rs") -Value "`npub fn changed_needles() -> &'static str { `"changed needle context`" }"
Set-Content -LiteralPath (Join-Path $repoRoot "src\diff.rs") -Value "pub fn new_diff_file() -> &'static str { `"new needle diff context`" }"
$rawBytesAfterDiff = Get-RawRepoBytes -RepoRoot $repoRoot

$diffResponses = Invoke-McpBatch -Requests @(
    New-BenchmarkRequest 10 "assemble_context" @{
        root = $repoRoot
        mode = "diff_aware"
        query = "needle"
        limit = 3
    }
)
$diffAware = Get-ToolPayload $diffResponses[0]

if ($diffAware.stale.is_stale -ne $true) {
    throw "diff_aware benchmark expected stale status after local file changes"
}

$metrics = @(
    New-BenchmarkMetric "exact_search" $rawBytesBeforeDiff $exactFirst
    New-BenchmarkMetric "overview" $rawBytesBeforeDiff $overview
    New-BenchmarkMetric "handoff_packet" $rawBytesBeforeDiff $handoff
    New-BenchmarkMetric "diff_aware" $rawBytesAfterDiff $diffAware
)

$summary = [ordered]@{
    workspace_root = $WorkspaceRoot
    repo_root = $repoRoot
    raw_repo_bytes_before_diff = $rawBytesBeforeDiff
    raw_repo_bytes_after_diff = $rawBytesAfterDiff
    cache_assertions = [ordered]@{
        exact_items_before = $cacheBefore.exact_search_cache.item_count
        exact_items_after_first = $cacheAfterFirst.exact_search_cache.item_count
        exact_items_after_second = $cacheAfterSecond.exact_search_cache.item_count
        repeated_exact_packet_bytes = Get-JsonByteCount $exactSecond
    }
    metrics = $metrics
    decision_memory_in_handoff = $handoff.memory_decisions.Count
    diff_aware_stale = $diffAware.stale
    note = "Local byte and approximate-token comparison only; not provider billing, model benchmarking, or guaranteed savings."
}

$summary | ConvertTo-Json -Depth 12
