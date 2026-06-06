import type { CliTruthPayload, RetrievalTruthPayload, ToolDescriptor } from './truth'

export type TruthSummaryItem = {
  label: string
  value: string
  detail: string
}

function formatSummaryList(values: string[]): string {
  return values.length > 0 ? values.join(', ') : 'none'
}

function reportedBoolean(value: boolean | undefined): string {
  if (typeof value !== 'boolean') {
    return 'not reported'
  }
  return value ? 'enabled' : 'disabled'
}

type CacheTruthPayload = NonNullable<RetrievalTruthPayload>['cache']

function cacheModeValue(cache: CacheTruthPayload): string {
  if (!cache) {
    return 'not reported'
  }

  const modeFlags = [
    cache.exact_search_enabled,
    cache.overview_enabled,
    cache.task_capsule_enabled
  ]
  const reportedModes = modeFlags.filter((value) => typeof value === 'boolean')
  if (reportedModes.length === 0) {
    return 'not reported'
  }

  const enabledModes = reportedModes.filter((value) => value)
  return `${enabledModes.length} modes`
}

export function toTruthSummaryItems(
  tools: ToolDescriptor[],
  retrieval: RetrievalTruthPayload,
  cli: CliTruthPayload
): TruthSummaryItem[] {
  const modes = Array.isArray(retrieval?.modes) ? retrieval.modes : []
  const cache = retrieval?.cache ?? null
  const firstCommand = Array.isArray(cli?.commands) ? cli.commands[0]?.command : undefined

  return [
    {
      label: 'Tools',
      value: String(tools.length),
      detail: 'MCP tools exposed by the backend truth payload.'
    },
    {
      label: 'Retrieval',
      value: modes.length > 0 ? `${modes.length} modes` : 'not reported',
      detail: modes.length > 0 ? formatSummaryList(modes) : 'No retrieval modes were reported by /truth.'
    },
    {
      label: 'Cache',
      value: cacheModeValue(cache),
      detail: cache
        ? `exact ${reportedBoolean(cache.exact_search_enabled)}; overview ${reportedBoolean(cache.overview_enabled)}; task ${reportedBoolean(cache.task_capsule_enabled)}; sync invalidation ${reportedBoolean(cache.sync_invalidates_caches)}`
        : 'No cache contract was reported by /truth.'
    },
    {
      label: 'Memory',
      value: reportedBoolean(retrieval?.durable_memory_enabled),
      detail: 'Durable memory availability as reported by /truth.'
    },
    {
      label: 'Budget',
      value: reportedBoolean(retrieval?.budget_estimate_enabled),
      detail: retrieval?.budget_estimate_unit ?? 'Budget estimate unit was not reported.'
    },
    {
      label: 'Entrypoint',
      value: reportedBoolean(cli?.local_entrypoint_enabled),
      detail: firstCommand ?? 'No CLI command was reported by /truth.'
    }
  ]
}
