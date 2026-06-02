export type ToolDescriptor = {
  name?: string
  description?: string
  inputSchema?: {
    required?: string[]
    properties?: Record<string, unknown>
  } | null
}

export type BackendProof = {
  id?: string
  command?: string
}

export type RetrievalTruthPayload = {
  modes?: string[]
  inclusion_reason_kinds?: string[]
  omission_reason_kinds?: string[]
  limits?: {
    max_context_items?: number
    max_snippet_pack_bytes?: number
    max_document_pack_bytes?: number
    max_memory_pack_bytes?: number
    max_history_runs?: number
  } | null
  durable_memory_enabled?: boolean
  cache?: {
    exact_search_enabled?: boolean
    overview_enabled?: boolean
    task_capsule_enabled?: boolean
    sync_invalidates_caches?: boolean
  } | null
} | null

export type RetrievalTruthItem = {
  state: string
  title: string
  description: string
}

export type TruthPageState = {
  fetchState: string
  fetchError: string
  backendTools: ToolDescriptor[]
  proofs: BackendProof[]
  retrievalTruth: RetrievalTruthItem[]
}

export function formatList(values: string[]): string {
  return values.length > 0 ? values.join(', ') : 'none'
}

export function toolSignals(tool: ToolDescriptor): string[] {
  const schema = tool?.inputSchema
  if (!schema || typeof schema !== 'object') {
    return ['no schema']
  }

  const required = Array.isArray(schema.required) ? schema.required : []
  const properties = schema.properties && typeof schema.properties === 'object'
    ? Object.keys(schema.properties)
    : []

  if (required.length === 0 && properties.length === 0) {
    return ['no input']
  }

  const chips: string[] = []
  if (required.length > 0) {
    chips.push(`required: ${required.join(', ')}`)
  }
  if (properties.length > 0) {
    chips.push(`fields: ${properties.join(', ')}`)
  }
  return chips
}

export function proofTitle(proof: BackendProof): string {
  return String(proof?.id ?? 'proof').replaceAll('_', ' ')
}

export function toRetrievalTruthItems(retrieval: RetrievalTruthPayload): RetrievalTruthItem[] {
  const limits = retrieval?.limits ?? {}
  return [
    {
      state: 'Modes',
      title: 'Retrieval modes',
      description: formatList(Array.isArray(retrieval?.modes) ? retrieval.modes : [])
    },
    {
      state: 'Inclusion',
      title: 'Inclusion reason kinds',
      description: formatList(Array.isArray(retrieval?.inclusion_reason_kinds) ? retrieval.inclusion_reason_kinds : [])
    },
    {
      state: 'Omission',
      title: 'Omission reason kinds',
      description: formatList(Array.isArray(retrieval?.omission_reason_kinds) ? retrieval.omission_reason_kinds : [])
    },
    {
      state: 'Limits',
      title: 'Hard limits',
      description: `items ${limits.max_context_items ?? 0}; snippet bytes ${limits.max_snippet_pack_bytes ?? 0}; document bytes ${limits.max_document_pack_bytes ?? 0}; memory bytes ${limits.max_memory_pack_bytes ?? 0}; history runs ${limits.max_history_runs ?? 0}`
    },
    {
      state: 'Memory',
      title: 'Durable memory',
      description: retrieval?.durable_memory_enabled ? 'enabled' : 'disabled'
    },
    {
      state: 'Cache',
      title: 'Cache state',
      description: retrieval?.cache
        ? `exact_search ${retrieval.cache.exact_search_enabled ? 'enabled' : 'disabled'}; overview ${retrieval.cache.overview_enabled ? 'enabled' : 'disabled'}; task_capsule ${retrieval.cache.task_capsule_enabled ? 'enabled' : 'disabled'}; sync invalidation ${retrieval.cache.sync_invalidates_caches ? 'enabled' : 'disabled'}`
        : 'none'
    },
    {
      state: 'Mirror',
      title: 'Truth source',
      description: 'Rendered from the live backend /truth payload.'
    }
  ]
}

export async function loadBackendTruth(
  fetchImpl: typeof fetch,
  truthRoute = '/truth'
): Promise<TruthPageState> {
  try {
    const response = await fetchImpl(truthRoute, {
      headers: {
        accept: 'application/json'
      }
    })

    if (!response.ok) {
      throw new Error(`GET ${truthRoute} returned ${response.status}`)
    }

    const payload = await response.json()
    const retrieval = payload?.retrieval ?? null
    const normalizedRetrieval = retrieval
      ? {
          ...retrieval,
          cache: retrieval.cache ?? payload?.cache ?? null,
        }
      : null

    return {
      fetchState: 'Live',
      fetchError: '',
      backendTools: Array.isArray(payload?.tools) ? payload.tools : [],
      proofs: Array.isArray(payload?.proofs) ? payload.proofs : [],
      retrievalTruth: toRetrievalTruthItems(normalizedRetrieval)
    }
  } catch (error) {
    return {
      fetchState: 'Unavailable',
      fetchError: error instanceof Error ? error.message : 'Failed to load /truth',
      backendTools: [],
      proofs: [],
      retrievalTruth: []
    }
  }
}