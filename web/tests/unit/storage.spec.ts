import { beforeEach, describe, expect, it } from 'vitest'
import { WebStore } from '@/lib/storage'

describe('WebStore', () => {
  beforeEach(() => {
    localStorage.clear()
  })

  it('persists values under its own namespace', async () => {
    const authStore = new WebStore('auth.json')
    const settingsStore = new WebStore('settings.json')

    await authStore.set('token', 'test-token')
    await settingsStore.set('theme', 'dark')

    expect(await authStore.get('token')).toBe('test-token')
    expect(await authStore.entries()).toEqual([['token', 'test-token']])
    expect(localStorage.getItem('gloamy:auth.json')).toBe('{"token":"test-token"}')
    expect(localStorage.getItem('gloamy:settings.json')).toBe('{"theme":"dark"}')
  })

  it('starts with an empty snapshot when persisted JSON is corrupt', async () => {
    localStorage.setItem('gloamy:auth.json', '{invalid')
    const store = new WebStore('auth.json')

    expect(await store.get('token')).toBeNull()
    await store.set('token', 'test-token')
    expect(localStorage.getItem('gloamy:auth.json')).toBe('{"token":"test-token"}')
  })
})
