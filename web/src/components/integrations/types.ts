import type { Component } from 'vue'

export type IntegrationSetupCapability = 'oauth_device_code'

export interface IntegrationCatalogEntry {
  id?: string
  name: string
  description: string
  category: string
  status: 'Available' | 'Active' | 'ComingSoon'
  setup_capability?: IntegrationSetupCapability | string
  connected?: boolean
}

export interface IntegrationSetupRegistration {
  capability: IntegrationSetupCapability
  component: Component
}

export type IntegrationSetupRegistry = Partial<Record<string, IntegrationSetupRegistration>>
