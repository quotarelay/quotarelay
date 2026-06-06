import { describe, expect, it, vi } from 'vitest'

import { loadBackendTruth, loadContextRunState, loadMemoryState, loadRepositoryState, proofTitle, toolSignals, toConfigTruthItems, toRetrievalTruthItems } from './truth'
import { toTruthSummaryItems } from './truthSummary'

describe('control-plane truth boundary', () => {
  it('normalizes the live /truth payload for the page', async () => {
    const fetchImpl = vi.fn(async () => ({
      ok: true,
      json: async () => ({
        tools: [
          {
            name: 'sync_repo',
            description: 'Sync a repository.',
            inputSchema: {
              required: ['root'],
              properties: {
                root: { type: 'string' },
                limit: { type: 'number' }
              }
            }
          }
        ],
        proofs: [{ id: 'tools_list', command: 'cargo test -p mcp-server tools_list_exposes_current_backend_truth_surface' }],
        retrieval: {
          modes: ['exact_search', 'overview', 'task_capsule', 'diff_aware'],
          inclusion_reason_kinds: ['query_line_match', 'diff_changed_file', 'diff_related_match'],
          omission_reason_kinds: ['byte_budget_reached', 'missing_indexed_file'],
          limits: {
            max_context_items: 5,
            max_snippet_pack_bytes: 160,
            max_document_pack_bytes: 640,
            max_memory_pack_bytes: 320,
            max_history_runs: 10
          },
          durable_memory_enabled: true,
          budget_estimate_enabled: true,
          budget_estimate_unit: 'approximate_tokens_from_included_bytes',
          cache: {
            exact_search_enabled: true,
            overview_enabled: true,
            task_capsule_enabled: true,
            sync_invalidates_caches: true
          }
        },
        config: {
          workspace_profiles_enabled: true,
          workspace_profiles_apply_to_retrieval: false,
          max_workspace_profiles: 20,
          max_profile_repo_roots: 5,
          default_mode: 'exact_search',
          default_limit: 3,
          per_repo_limit: 3
        },
        cli: {
          local_entrypoint_enabled: true,
          commands: [
            {
              label: 'Truth',
              command: 'cargo run -p mcp-server -- --cli truth'
            }
          ]
        }
      })
    })) as typeof fetch

    const state = await loadBackendTruth(fetchImpl, '/truth')

    expect(fetchImpl).toHaveBeenCalledWith('/truth', {
      headers: {
        accept: 'application/json'
      }
    })
    expect(state.fetchState).toBe('Live')
    expect(state.fetchError).toBe('')
    expect(state.backendTools).toHaveLength(1)
    expect(toolSignals(state.backendTools[0])).toEqual(['required: root', 'fields: root, limit'])
    expect(proofTitle(state.proofs[0])).toBe('tools list')
    expect(state.truthSummary).toEqual(toTruthSummaryItems(state.backendTools, {
      modes: ['exact_search', 'overview', 'task_capsule', 'diff_aware'],
      inclusion_reason_kinds: ['query_line_match', 'diff_changed_file', 'diff_related_match'],
      omission_reason_kinds: ['byte_budget_reached', 'missing_indexed_file'],
      limits: {
        max_context_items: 5,
        max_snippet_pack_bytes: 160,
        max_document_pack_bytes: 640,
        max_memory_pack_bytes: 320,
        max_history_runs: 10
      },
      durable_memory_enabled: true,
      budget_estimate_enabled: true,
      budget_estimate_unit: 'approximate_tokens_from_included_bytes',
      cache: {
        exact_search_enabled: true,
        overview_enabled: true,
        task_capsule_enabled: true,
        sync_invalidates_caches: true
      }
    }, {
      local_entrypoint_enabled: true,
      commands: [
        {
          label: 'Truth',
          command: 'cargo run -p mcp-server -- --cli truth'
        }
      ]
    }))
    expect(state.retrievalTruth).toEqual(toRetrievalTruthItems({
      modes: ['exact_search', 'overview', 'task_capsule', 'diff_aware'],
      inclusion_reason_kinds: ['query_line_match', 'diff_changed_file', 'diff_related_match'],
      omission_reason_kinds: ['byte_budget_reached', 'missing_indexed_file'],
      limits: {
        max_context_items: 5,
        max_snippet_pack_bytes: 160,
        max_document_pack_bytes: 640,
        max_memory_pack_bytes: 320,
        max_history_runs: 10
      },
      durable_memory_enabled: true,
      budget_estimate_enabled: true,
      budget_estimate_unit: 'approximate_tokens_from_included_bytes',
      cache: {
        exact_search_enabled: true,
        overview_enabled: true,
        task_capsule_enabled: true,
        sync_invalidates_caches: true
      }
    }))
    expect(state.configTruth).toEqual(toConfigTruthItems({
      workspace_profiles_enabled: true,
      workspace_profiles_apply_to_retrieval: false,
      max_workspace_profiles: 20,
      max_profile_repo_roots: 5,
      default_mode: 'exact_search',
      default_limit: 3,
      per_repo_limit: 3
    }))
    expect(state.cliCommands).toEqual([
      {
        label: 'Truth',
        command: 'cargo run -p mcp-server -- --cli truth'
      }
    ])
  })

  it('falls back honestly when /truth is unavailable', async () => {
    const fetchImpl = vi.fn(async () => {
      throw new Error('network down')
    }) as typeof fetch

    const state = await loadBackendTruth(fetchImpl, '/truth')

    expect(state).toEqual({
      fetchState: 'Unavailable',
      fetchError: 'network down',
      backendTools: [],
      proofs: [],
      truthSummary: [],
      retrievalTruth: [],
      configTruth: [],
      cliCommands: []
    })
  })

  it('summarizes only reported backend truth fields', () => {
    const summary = toTruthSummaryItems([], {
      modes: [],
      durable_memory_enabled: undefined,
      budget_estimate_enabled: false,
      cache: null
    }, null)

    expect(summary).toEqual([
      {
        label: 'Tools',
        value: '0',
        detail: 'MCP tools exposed by the backend truth payload.'
      },
      {
        label: 'Retrieval',
        value: 'not reported',
        detail: 'No retrieval modes were reported by /truth.'
      },
      {
        label: 'Cache',
        value: 'not reported',
        detail: 'No cache contract was reported by /truth.'
      },
      {
        label: 'Memory',
        value: 'not reported',
        detail: 'Durable memory availability as reported by /truth.'
      },
      {
        label: 'Budget',
        value: 'disabled',
        detail: 'Budget estimate unit was not reported.'
      },
      {
        label: 'Entrypoint',
        value: 'not reported',
        detail: 'No CLI command was reported by /truth.'
      }
    ])
  })

  it('summarizes cache modes separately from sync invalidation', () => {
    const summary = toTruthSummaryItems([], {
      modes: ['exact_search'],
      cache: {
        exact_search_enabled: true,
        overview_enabled: false,
        task_capsule_enabled: true,
        sync_invalidates_caches: false
      }
    }, null)

    expect(summary.find((item) => item.label === 'Cache')).toEqual({
      label: 'Cache',
      value: '2 modes',
      detail: 'exact enabled; overview disabled; task enabled; sync invalidation disabled'
    })
  })

  it('normalizes cache state when /truth exposes it at the root payload level', async () => {
    const fetchImpl = vi.fn(async () => ({
      ok: true,
      json: async () => ({
        tools: [],
        proofs: [],
        retrieval: {
          modes: ['exact_search'],
          inclusion_reason_kinds: ['query_line_match'],
          omission_reason_kinds: ['byte_budget_reached'],
          limits: {
            max_context_items: 5,
            max_snippet_pack_bytes: 160,
            max_document_pack_bytes: 640,
            max_memory_pack_bytes: 320,
            max_history_runs: 10
          },
          durable_memory_enabled: true,
          budget_estimate_enabled: true,
          budget_estimate_unit: 'approximate_tokens_from_included_bytes'
        },
        cache: {
          exact_search_enabled: true,
          overview_enabled: false,
          task_capsule_enabled: false,
          sync_invalidates_caches: true
        }
      })
    })) as typeof fetch

    const state = await loadBackendTruth(fetchImpl, '/truth')

    expect(state.retrievalTruth).toEqual(toRetrievalTruthItems({
      modes: ['exact_search'],
      inclusion_reason_kinds: ['query_line_match'],
      omission_reason_kinds: ['byte_budget_reached'],
      limits: {
        max_context_items: 5,
        max_snippet_pack_bytes: 160,
        max_document_pack_bytes: 640,
        max_memory_pack_bytes: 320,
        max_history_runs: 10
      },
      durable_memory_enabled: true,
      budget_estimate_enabled: true,
      budget_estimate_unit: 'approximate_tokens_from_included_bytes',
      cache: {
        exact_search_enabled: true,
        overview_enabled: false,
        task_capsule_enabled: false,
        sync_invalidates_caches: true
      }
    }))
  })

  it('loads repository state only when a root is configured', async () => {
    const fetchImpl = vi.fn(async () => ({
      ok: true,
      json: async () => ([
        {
          repository: {
            name: 'quotarelay',
            root: 'C:\\repo\\quotarelay'
          },
          sync: {
            status: 'indexed',
            indexed_files: 4
          },
          recent_context_run: {
            query: 'needle',
            generated_at_epoch_ms: 42
          },
          repo_map: {
            directories: [
              { path: 'src', indexed_files: 3 },
              { path: '.', indexed_files: 1 }
            ],
            rust_files: [
              { path: 'src/lib.rs', symbols: ['pub mod api;', 'pub struct Widget;'] }
            ],
            omitted_directory_count: 0,
            omitted_rust_file_count: 0
          }
        }
      ])
    })) as typeof fetch

    const state = await loadRepositoryState(fetchImpl, 'C:\\repo\\quotarelay')

    expect(fetchImpl).toHaveBeenCalledWith('/repositories?root=C%3A%5Crepo%5Cquotarelay&limit=20', {
      headers: {
        accept: 'application/json'
      }
    })
    expect(state.fetchState).toBe('Live')
    expect(state.repositories).toHaveLength(1)
    expect(state.repositories[0].sync?.status).toBe('indexed')
    expect(state.repositories[0].repo_map?.directories?.[0].path).toBe('src')
    expect(state.repositories[0].repo_map?.rust_files?.[0].symbols?.[1]).toContain('Widget')
  })

  it('does not invent repository state without a configured root', async () => {
    const fetchImpl = vi.fn() as unknown as typeof fetch

    const state = await loadRepositoryState(fetchImpl, '')

    expect(fetchImpl).not.toHaveBeenCalled()
    expect(state).toEqual({
      fetchState: 'Not configured',
      fetchError: '',
      repositories: []
    })
  })

  it('loads bounded memory state only when root and query are configured', async () => {
    const fetchImpl = vi.fn(async () => ({
      ok: true,
      json: async () => ({
        notes: [{ id: '1', title: 'Needle note', content: 'needle memory', tags: ['memory'] }],
        omitted_count: 0
      })
    })) as typeof fetch

    const state = await loadMemoryState(fetchImpl, 'C:\\repo\\quotarelay', 'needle')

    expect(fetchImpl).toHaveBeenCalledWith('/memory?root=C%3A%5Crepo%5Cquotarelay&query=needle&limit=3', {
      headers: {
        accept: 'application/json'
      }
    })
    expect(state.fetchState).toBe('Live')
    expect(state.memory.notes).toHaveLength(1)
    expect(state.memory.notes[0].title).toBe('Needle note')
  })

  it('does not invent memory state without root and query', async () => {
    const fetchImpl = vi.fn() as unknown as typeof fetch

    const state = await loadMemoryState(fetchImpl, '', '')

    expect(fetchImpl).not.toHaveBeenCalled()
    expect(state).toEqual({
      fetchState: 'Not configured',
      fetchError: '',
      memory: {
        notes: [],
        omitted_count: 0
      }
    })
  })

  it('loads bounded context run state only when root is configured', async () => {
    const fetchImpl = vi.fn(async () => ({
      ok: true,
      json: async () => ([
        {
          query: 'needle',
          generated_at_epoch_ms: 42,
          budget: {
            raw_bytes_considered: 120,
            included_bytes: 12,
            approximate_tokens: 3,
            estimated_reduction_ratio: 0.9
          },
          stale: {
            is_stale: true,
            changed_files: 1,
            missing_files: 0,
            new_files: 1
          },
          cache_status: {
            kind: 'hit',
            detail: 'exact_search cache entry returned this context pack.'
          },
          snippets: [{ path: 'alpha.txt', reason: { kind: 'query_line_match' } }],
          omissions: [{ kind: 'item_limit_reached' }]
        }
      ])
    })) as typeof fetch

    const state = await loadContextRunState(fetchImpl, 'C:\\repo\\quotarelay')

    expect(fetchImpl).toHaveBeenCalledWith('/context-runs?root=C%3A%5Crepo%5Cquotarelay&limit=5', {
      headers: {
        accept: 'application/json'
      }
    })
    expect(state.fetchState).toBe('Live')
    expect(state.runs).toHaveLength(1)
    expect(state.runs[0].budget?.raw_bytes_considered).toBe(120)
    expect(state.runs[0].budget?.approximate_tokens).toBe(3)
    expect(state.runs[0].budget?.estimated_reduction_ratio).toBe(0.9)
    expect(state.runs[0].stale?.is_stale).toBe(true)
    expect(state.runs[0].cache_status?.kind).toBe('hit')
    expect(state.runs[0].omissions?.[0].kind).toBe('item_limit_reached')
  })
})
