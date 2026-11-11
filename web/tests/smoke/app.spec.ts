import { expect, test } from '@playwright/test'

test('shows the pairing screen when no token is stored', async ({ page }) => {
  await page.route('**/health', route => route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({ paired: false }),
  }))

  await page.goto('/')

  await expect(page).toHaveURL(/#\/authentication(?:\?.*)?$/)
  await expect(page.getByRole('heading', { name: 'Authentication' })).toBeVisible()
  await expect(page.getByText('Pairing code')).toBeVisible()
})

test('loads the authenticated application shell with daemon requests mocked', async ({ page }) => {
  await page.addInitScript(() => {
    localStorage.setItem('gloamy:auth.json', JSON.stringify({ token: 'test-token' }))
  })

  await page.route('**/health', route => route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({ paired: true }),
  }))

  await page.route('**/api/**', async (route) => {
    const pathname = new URL(route.request().url()).pathname
    const body = pathname.endsWith('/status')
      ? {
          provider: 'test-provider',
          model: 'test-model',
          temperature: 0.2,
          uptime_seconds: 1,
          gateway_port: 42617,
          locale: 'en',
          memory_backend: 'markdown',
          paired: true,
          channels: {},
          health: { pid: 1, updated_at: '', uptime_seconds: 1, components: {} },
        }
      : pathname.endsWith('/cost')
        ? {
            session_cost_usd: 0,
            daily_cost_usd: 0,
            monthly_cost_usd: 0,
            total_tokens: 0,
            request_count: 0,
            by_model: {},
          }
        : { timeline: [] }

    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(body),
    })
  })

  await page.goto('/')

  await expect(page.getByText('Runtime Overview')).toBeVisible()
  await expect(page.getByText('Dashboard', { exact: true })).toBeVisible()
})
