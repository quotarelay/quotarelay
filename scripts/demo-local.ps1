param(
    [string]$WorkspaceRoot = ""
)

$ErrorActionPreference = "Stop"

function New-DemoRequest {
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
        $json = $request | ConvertTo-Json -Depth 20 -Compress
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

if ([string]::IsNullOrWhiteSpace($WorkspaceRoot)) {
    $WorkspaceRoot = Join-Path ([IO.Path]::GetTempPath()) ("quotarelay-demo-" + [Guid]::NewGuid())
}

$repoRoot = Join-Path $WorkspaceRoot "repo"
$stateRoot = Join-Path $WorkspaceRoot "state"
New-Item -ItemType Directory -Force -Path $WorkspaceRoot, $stateRoot | Out-Null
Copy-Item -Recurse -Force -Path "examples\demo-repo" -Destination $repoRoot

$requests = @(
    New-DemoRequest 1 "register_repository" @{
        root = $stateRoot
        repo_root = $repoRoot
    }
    New-DemoRequest 2 "sync_repo" @{
        root = $repoRoot
    }
    New-DemoRequest 3 "repo_inventory" @{
        root = $repoRoot
    }
    New-DemoRequest 4 "memory_write" @{
        root = $repoRoot
        title = "Demo decision"
        content = "The demo needle workflow should prefer bounded local context."
        tags = @("demo", "needle")
    }
    New-DemoRequest 5 "memory_search" @{
        root = $repoRoot
        query = "needle"
        limit = 3
    }
    New-DemoRequest 6 "assemble_context" @{
        root = $repoRoot
        mode = "exact_search"
        query = "needle"
        limit = 2
    }
    New-DemoRequest 7 "assemble_context" @{
        root = $repoRoot
        mode = "exact_search"
        query = "needle"
        limit = 2
    }
    New-DemoRequest 8 "handoff_packet" @{
        root = $repoRoot
        active_task = "Continue the local demo needle workflow"
        template = "feature_slice"
        mode = "exact_search"
        query = "needle"
        limit = 2
    }
    New-DemoRequest 9 "assemble_context" @{
        root = $repoRoot
        mode = "overview"
        limit = 2
    }
    New-DemoRequest 10 "cache_inspect" @{
        root = $repoRoot
    }
)

$responses = Invoke-McpBatch -Requests $requests
$registered = Get-ToolPayload $responses[0]
$syncStatus = $responses[1].result.content[0].text
$inventory = Get-ToolPayload $responses[2]
$memoryWrite = Get-ToolPayload $responses[3]
$memorySearch = Get-ToolPayload $responses[4]
$exact = Get-ToolPayload $responses[5]
$exactRepeat = Get-ToolPayload $responses[6]
$handoff = Get-ToolPayload $responses[7]
$overview = Get-ToolPayload $responses[8]
$cache = Get-ToolPayload $responses[9]
$truth = cargo run -p mcp-server -- --cli truth | ConvertFrom-Json

$summary = [ordered]@{
    workspace_root = $WorkspaceRoot
    registered_repo = $registered.repository.root
    sync_status = $syncStatus
    indexed_files = $inventory.indexed_files
    memory_note = $memoryWrite.note.title
    memory_search_count = $memorySearch.notes.Count
    exact_mode = $exact.mode
    exact_snippets = $exact.snippets.Count
    exact_memory_notes = $exact.memory_notes.Count
    first_exact_cache_status = $exact.cache_status.kind
    repeated_exact_cache_status = $exactRepeat.cache_status.kind
    handoff_template = $handoff.template
    handoff_validation_commands = $handoff.validation_commands.Count
    overview_documents = $overview.documents.Count
    cache_exact_items = $cache.exact_search_cache.item_count
    cache_capsule_items = $cache.retrieval_capsule_cache.item_count
    truth_tool_count = $truth.result.truth.tools.Count
    provider_calls = "none"
}

$summary | ConvertTo-Json -Depth 6
