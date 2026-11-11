import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { apiFetch, pair, UnauthorizedError } from '@/lib/api'

const fetchMock = vi.fn<typeof fetch>()

describe('apiFetch', () => {
  beforeEach(() => {
    fetchMock.mockReset()
    vi.stubGlobal('fetch', fetchMock)
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('normalizes URLs and applies bearer and JSON headers', async () => {
    fetchMock.mockResolvedValueOnce(new Response('{"ok":true}', { status: 200 }))

    const result = await apiFetch<{ ok: boolean }>(
      ' https://daemon.example/ ',
      'api/status',
      { method: 'POST', body: '{}' },
      'test-token',
    )

    expect(result).toEqual({ ok: true })
    const [url, options] = fetchMock.mock.calls[0]
    const headers = new Headers(options?.headers)
    expect(url).toBe('https://daemon.example/api/status')
    expect(headers.get('Authorization')).toBe('Bearer test-token')
    expect(headers.get('Content-Type')).toBe('application/json')
  })

  it('throws UnauthorizedError for HTTP 401', async () => {
    fetchMock.mockResolvedValueOnce(new Response(null, { status: 401 }))

    await expect(apiFetch('', '/api/status')).rejects.toBeInstanceOf(UnauthorizedError)
  })

  it('includes the response body in non-authorization errors', async () => {
    fetchMock.mockResolvedValueOnce(new Response('daemon unavailable', { status: 503 }))

    await expect(apiFetch('', '/api/status')).rejects.toThrow('API 503: daemon unavailable')
  })

  it('returns undefined for no-content responses', async () => {
    fetchMock.mockResolvedValueOnce(new Response(null, { status: 204 }))

    await expect(apiFetch('', '/api/status')).resolves.toBeUndefined()
  })

  it('sends pairing codes in the pairing request header', async () => {
    fetchMock.mockResolvedValueOnce(new Response('{"token":"test-token"}', { status: 200 }))

    await expect(pair('https://daemon.example/', '123456')).resolves.toEqual({ token: 'test-token' })
    expect(fetchMock).toHaveBeenCalledWith('https://daemon.example/pair', {
      method: 'POST',
      headers: { 'X-Pairing-Code': '123456' },
    })
  })
})
