export type SurfaceStatus = 'available' | 'planned' | 'deferred'

export type DeploymentSurface = {
  id: string
  title: string
  status: SurfaceStatus
  target: string
  localPreview: string
  deploymentBoundary: string
}

export const deploymentSurfaces: DeploymentSurface[] = [
  {
    id: 'public-site',
    title: 'Public site',
    status: 'available',
    target: 'DOM',
    localPreview: 'Vite preview at /',
    deploymentBoundary: 'Static web hosting; no backend state required.'
  },
  {
    id: 'local-control-plane',
    title: 'Local control plane',
    status: 'available',
    target: 'DOM + loopback backend',
    localPreview: 'Vite preview at /control-plane with local /truth backend',
    deploymentBoundary: 'Local operator UI only; do not expose the loopback backend publicly.'
  },
  {
    id: 'local-engine',
    title: 'Local MCP engine',
    status: 'available',
    target: 'stdio MCP + CLI',
    localPreview: 'cargo run -p mcp-server -- --cli truth or installed quotarelay-mcp --cli truth',
    deploymentBoundary: 'Agent-facing local MCP tool; no native desktop/mobile app or hosted tenancy.'
  },
  {
    id: 'hosted-team-console',
    title: 'Hosted team console',
    status: 'planned',
    target: 'DOM + hosted API',
    localPreview: 'Design and threat-model slice before implementation',
    deploymentBoundary: 'Requires auth, tenant isolation, audit, retention, and privacy controls.'
  },
  {
    id: 'private-deployment',
    title: 'Private deployment',
    status: 'planned',
    target: 'container or private package',
    localPreview: 'Deployment decision doc before container build',
    deploymentBoundary: 'Requires signing, rollout, rollback, monitoring, backup, and support policy.'
  },
  {
    id: 'native-companion',
    title: 'Native desktop and mobile apps',
    status: 'deferred',
    target: 'Desktop / Android / iOS',
    localPreview: 'Not shipped; evaluate only if users need a companion wrapper around the MCP engine',
    deploymentBoundary: 'Must not become the primary product or imply source sync, hosted tenancy, or local secret storage without a threat model.'
  }
]

export function surfaceStatusLabel(status: SurfaceStatus): string {
  switch (status) {
    case 'available':
      return 'Available locally'
    case 'planned':
      return 'Planned'
    case 'deferred':
      return 'Deferred'
  }
}
