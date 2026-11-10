import type { IntegrationSetupRegistry } from './types'
import CodexSetupDialog from './CodexSetupDialog.vue'

/** Setup dialogs are registered by stable integration ID. */
export const integrationSetupRegistry: IntegrationSetupRegistry = {
  'openai-codex': {
    capability: 'oauth_device_code',
    component: CodexSetupDialog,
  },
}
