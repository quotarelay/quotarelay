import { describe, expect, it, vi } from 'vitest'

import { loadBackendTruth, proofTitle, toolSignals, toRetrievalTruthItems } from './truth'

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
          modes: ['exact_search', 'overview', 'task_capsule'],
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
          cache: {
            exact_search_enabled: true,
            overview_enabled: true,
            task_capsule_enabled: true,
            sync_invalidates_caches: true
          }
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
    expect(state.retrievalTruth).toEqual(toRetrievalTruthItems({
      modes: ['exact_search', 'overview', 'task_capsule'],
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
      cache: {
        exact_search_enabled: true,
        overview_enabled: true,
        task_capsule_enabled: true,
        sync_invalidates_caches: true
      }
    }))
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
      retrievalTruth: []
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
          durable_memory_enabled: true
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
      cache: {
        exact_search_enabled: true,
        overview_enabled: false,
        task_capsule_enabled: false,
        sync_invalidates_caches: true
      }
    }))
  })
})