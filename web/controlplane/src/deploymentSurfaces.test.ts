import { describe, expect, it } from 'vitest'

import { deploymentSurfaces, surfaceStatusLabel } from './deploymentSurfaces'

describe('deployment surface planning model', () => {
  it('separates available local surfaces from planned and deferred deployment work', () => {
    expect(deploymentSurfaces.map((surface) => surface.id)).toEqual([
      'public-site',
      'local-control-plane',
      'local-engine',
      'hosted-team-console',
      'private-deployment',
      'native-companion'
    ])

    expect(deploymentSurfaces.filter((surface) => surface.status === 'available').map((surface) => surface.id)).toEqual([
      'public-site',
      'local-control-plane',
      'local-engine'
    ])
    expect(deploymentSurfaces.find((surface) => surface.id === 'native-companion')?.status).toBe('deferred')
    expect(deploymentSurfaces.find((surface) => surface.id === 'local-engine')?.localPreview).toContain('quotarelay-mcp')
  })

  it('keeps deployment boundaries explicit and privacy-first', () => {
    for (const surface of deploymentSurfaces) {
      expect(surface.deploymentBoundary.length).toBeGreaterThan(20)
      expect(surface.localPreview.length).toBeGreaterThan(10)
    }

    expect(deploymentSurfaces.find((surface) => surface.id === 'local-control-plane')?.deploymentBoundary).toContain('do not expose')
    expect(deploymentSurfaces.find((surface) => surface.id === 'local-engine')?.deploymentBoundary).toContain('no native desktop/mobile app')
    expect(deploymentSurfaces.find((surface) => surface.id === 'hosted-team-console')?.deploymentBoundary).toContain('privacy controls')
    expect(surfaceStatusLabel('available')).toBe('Available locally')
    expect(surfaceStatusLabel('planned')).toBe('Planned')
    expect(surfaceStatusLabel('deferred')).toBe('Deferred')
  })
})
