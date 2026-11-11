import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const fetchMock = vi.fn<typeof fetch>()

async function createAuthStore() {
  vi.resetModules()
  const { createPinia, setActivePinia } = await import('pinia')
  const { useAuthStore } = await import('@/stores/auth')
  setActivePinia(createPinia())
  return useAuthStore()
}

describe('auth store', () => {
  beforeEach(() => {
    localStorage.clear()
    fetchMock.mockReset()
    vi.stubGlobal('fetch', fetchMock)
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('loads a stored token and validates it with the daemon', async () => {
    localStorage.setItem('gloamy:auth.json', JSON.stringify({ token: 'test-token' }))
    fetchMock.mockResolvedValueOnce(new Response('{"status":"ok"}', { status: 200 }))
    const auth = await createAuthStore()

    await auth.load()

    expect(auth.isLoaded).toBe(true)
    expect(auth.isAuthenticated).toBe(true)
    expect(auth.authState).toBe('authenticated')
    const [, options] = fetchMock.mock.calls[0]
    expect(new Headers(options?.headers).get('Authorization')).toBe('Bearer test-token')
  })

  it('clears a rejected token and completes loading as unauthenticated', async () => {
    localStorage.setItem('gloamy:auth.json', JSON.stringify({ token: 'expired-token' }))
    fetchMock.mockResolvedValueOnce(new Response(null, { status: 401 }))
    const auth = await createAuthStore()

    await auth.load()

    expect(auth.isLoaded).toBe(true)
    expect(auth.isAuthenticated).toBe(false)
    expect(auth.authState).toBe('unauthenticated')
    expect(auth.token).toBeNull()
    expect(JSON.parse(localStorage.getItem('gloamy:auth.json') ?? '{}')).toEqual({ token: '' })
  })

  it('does not request validation when no token is stored', async () => {
    const auth = await createAuthStore()

    await auth.load()

    expect(auth.authState).toBe('unauthenticated')
    expect(fetchMock).not.toHaveBeenCalled()
  })
})
