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

export type RepositoryStatePayload = {
  repository?: {
    name?: string
    root?: string
  }
  sync?: {
    status?: string
    indexed_files?: number
  }
  recent_context_run?: {
    query?: string
    generated_at_epoch_ms?: number
  } | null
  repo_map?: {
    directories?: Array<{
      path?: string
      indexed_files?: number
    }>
    rust_files?: Array<{
      path?: string
      symbols?: string[]
    }>
    omitted_directory_count?: number
    omitted_rust_file_count?: number
  } | null
}

export type MemorySearchState = {
  notes: Array<{
    id?: string
    title?: string
    content?: string
    tags?: string[]
    profile?: string
  }>
  omitted_count: number
}

export type ContextRunState = {
  query?: string
  generated_at_epoch_ms?: number
  budget?: {
    included_bytes?: number
    approximate_tokens?: number
  }
  stale?: {
    is_stale?: boolean
    changed_files?: number
    missing_files?: number
    new_files?: number
  }
  snippets?: Array<{
    path?: string
    reason?: {
      kind?: string
    }
  }>
  omissions?: Array<{
    kind?: string
  }>
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
  budget_estimate_enabled?: boolean
  budget_estimate_unit?: string
  cache?: {
    exact_search_enabled?: boolean
    overview_enabled?: boolean
    task_capsule_enabled?: boolean
    sync_invalidates_caches?: boolean
  } | null
} | null

export type ConfigTruthPayload = {
  workspace_profiles_enabled?: boolean
  workspace_profiles_apply_to_retrieval?: boolean
  max_workspace_profiles?: number
  max_profile_repo_roots?: number
  default_mode?: string
  default_limit?: number
  per_repo_limit?: number
} | null

export type CliTruthPayload = {
  local_entrypoint_enabled?: boolean
  commands?: Array<{
    label?: string
    command?: string
  }>
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
  configTruth: RetrievalTruthItem[]
  cliCommands: Array<{
    label?: string
    command?: string
  }>
}

export type RepositoryPageState = {
  fetchState: string
  fetchError: string
  repositories: RepositoryStatePayload[]
}

export type MemoryPageState = {
  fetchState: string
  fetchError: string
  memory: MemorySearchState
}

export type ContextRunPageState = {
  fetchState: string
  fetchError: string
  runs: ContextRunState[]
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
      state: 'Estimate',
      title: 'Context budget estimate',
      description: retrieval?.budget_estimate_enabled
        ? `${retrieval.budget_estimate_unit ?? 'approximate_tokens_from_included_bytes'}`
        : 'disabled'
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

export function toConfigTruthItems(config: ConfigTruthPayload): RetrievalTruthItem[] {
  return [
    {
      state: 'Profiles',
      title: 'Workspace profiles',
      description: config?.workspace_profiles_enabled ? 'enabled' : 'disabled'
    },
    {
      state: 'Application',
      title: 'Retrieval override',
      description: config?.workspace_profiles_apply_to_retrieval ? 'applied' : 'not applied'
    },
    {
      state: 'Defaults',
      title: 'Built-in defaults',
      description: `mode ${config?.default_mode ?? 'unknown'}; default limit ${config?.default_limit ?? 0}; per-repo limit ${config?.per_repo_limit ?? 0}`
    },
    {
      state: 'Bounds',
      title: 'Profile bounds',
      description: `profiles ${config?.max_workspace_profiles ?? 0}; repo roots ${config?.max_profile_repo_roots ?? 0}`
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
      retrievalTruth: toRetrievalTruthItems(normalizedRetrieval),
      configTruth: toConfigTruthItems(payload?.config ?? null),
      cliCommands: Array.isArray(payload?.cli?.commands) ? payload.cli.commands : []
    }
  } catch (error) {
    return {
      fetchState: 'Unavailable',
      fetchError: error instanceof Error ? error.message : 'Failed to load /truth',
      backendTools: [],
      proofs: [],
      retrievalTruth: [],
      configTruth: [],
      cliCommands: []
    }
  }
}

export async function loadRepositoryState(
  fetchImpl: typeof fetch,
  root: string,
  repositoryRoute = '/repositories'
): Promise<RepositoryPageState> {
  if (root.trim() === '') {
    return {
      fetchState: 'Not configured',
      fetchError: '',
      repositories: []
    }
  }

  try {
    const response = await fetchImpl(`${repositoryRoute}?root=${encodeURIComponent(root)}&limit=20`, {
      headers: {
        accept: 'application/json'
      }
    })

    if (!response.ok) {
      throw new Error(`GET ${repositoryRoute} returned ${response.status}`)
    }

    const payload = await response.json()
    return {
      fetchState: 'Live',
      fetchError: '',
      repositories: Array.isArray(payload) ? payload : []
    }
  } catch (error) {
    return {
      fetchState: 'Unavailable',
      fetchError: error instanceof Error ? error.message : 'Failed to load /repositories',
      repositories: []
    }
  }
}

export async function loadMemoryState(
  fetchImpl: typeof fetch,
  root: string,
  query: string,
  memoryRoute = '/memory'
): Promise<MemoryPageState> {
  const empty = {
    notes: [],
    omitted_count: 0
  }
  if (root.trim() === '' || query.trim() === '') {
    return {
      fetchState: 'Not configured',
      fetchError: '',
      memory: empty
    }
  }

  try {
    const response = await fetchImpl(`${memoryRoute}?root=${encodeURIComponent(root)}&query=${encodeURIComponent(query)}&limit=3`, {
      headers: {
        accept: 'application/json'
      }
    })

    if (!response.ok) {
      throw new Error(`GET ${memoryRoute} returned ${response.status}`)
    }

    const payload = await response.json()
    return {
      fetchState: 'Live',
      fetchError: '',
      memory: {
        notes: Array.isArray(payload?.notes) ? payload.notes : [],
        omitted_count: Number(payload?.omitted_count ?? 0)
      }
    }
  } catch (error) {
    return {
      fetchState: 'Unavailable',
      fetchError: error instanceof Error ? error.message : 'Failed to load /memory',
      memory: empty
    }
  }
}

export async function loadContextRunState(
  fetchImpl: typeof fetch,
  root: string,
  runsRoute = '/context-runs'
): Promise<ContextRunPageState> {
  if (root.trim() === '') {
    return {
      fetchState: 'Not configured',
      fetchError: '',
      runs: []
    }
  }

  try {
    const response = await fetchImpl(`${runsRoute}?root=${encodeURIComponent(root)}&limit=5`, {
      headers: {
        accept: 'application/json'
      }
    })

    if (!response.ok) {
      throw new Error(`GET ${runsRoute} returned ${response.status}`)
    }

    const payload = await response.json()
    return {
      fetchState: 'Live',
      fetchError: '',
      runs: Array.isArray(payload) ? payload : []
    }
  } catch (error) {
    return {
      fetchState: 'Unavailable',
      fetchError: error instanceof Error ? error.message : 'Failed to load /context-runs',
      runs: []
    }
  }
}
