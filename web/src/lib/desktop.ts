import { invoke, isTauri } from '@tauri-apps/api/core'

export interface DesktopRuntime {
  ready: boolean
  api_base_url: string | null
  desktop_managed: boolean
  error: string | null
}

export const isDesktopApp = isTauri()

export function getDaemonStatus(): Promise<DesktopRuntime> {
  return invoke<DesktopRuntime>('get_daemon_status')
}

export function requestPairingCode(): Promise<string> {
  return invoke<string>('request_pairing_code')
}

export function getKeepDaemonRunning(): Promise<boolean> {
  return invoke<boolean>('get_keep_daemon_running')
}

export function setKeepDaemonRunning(keepRunning: boolean): Promise<void> {
  return invoke<void>('set_keep_daemon_running', { keepRunning })
}
