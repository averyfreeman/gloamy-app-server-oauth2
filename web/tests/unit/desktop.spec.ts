import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { clearMocks, mockIPC } from '@tauri-apps/api/mocks'

const fetchMock = vi.fn<typeof fetch>()

async function createAuthStore() {
  vi.resetModules()
  const { createPinia, setActivePinia } = await import('pinia')
  const { useAuthStore } = await import('@/stores/auth')
  setActivePinia(createPinia())
  return useAuthStore()
}

describe('desktop runtime initialization', () => {
  beforeEach(() => {
    localStorage.clear()
    fetchMock.mockReset()
    vi.stubGlobal('fetch', fetchMock)
    Reflect.deleteProperty(globalThis, 'isTauri')
    clearMocks()
  })

  afterEach(() => {
    clearMocks()
    Reflect.deleteProperty(globalThis, 'isTauri')
    vi.unstubAllGlobals()
  })

  it('uses the native daemon address before validating a stored token', async () => {
    localStorage.setItem('gloamy:auth.json', JSON.stringify({
      token: 'desktop-token',
      api_base_url: 'http://127.0.0.9:1',
    }))
    Reflect.set(globalThis, 'isTauri', true)
    mockIPC((command) => {
      if (command === 'get_daemon_status') {
        return {
          ready: true,
          api_base_url: 'http://127.0.0.1:42617',
          desktop_managed: true,
          error: null,
        }
      }
      throw new Error(`Unexpected Tauri command: ${command}`)
    })
    fetchMock.mockResolvedValueOnce(new Response('{"status":"ok"}', { status: 200 }))

    const auth = await createAuthStore()
    await auth.load()

    expect(auth.isDesktopApp).toBe(true)
    expect(auth.desktopRuntime?.ready).toBe(true)
    expect(fetchMock.mock.calls[0]?.[0]).toBe('http://127.0.0.1:42617/api/status')
  })

  it('keeps browser mode on its existing same-origin path', async () => {
    const auth = await createAuthStore()

    await auth.load()

    expect(auth.isDesktopApp).toBe(false)
    expect(auth.desktopRuntime).toBeNull()
    expect(auth.baseUrl).toBe('')
    expect(fetchMock).not.toHaveBeenCalled()
  })
})
