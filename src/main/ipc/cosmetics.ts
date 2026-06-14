import { IpcMain } from 'electron'
import http from 'http'
import { getBejaToken } from '../services/bejaAuth'

const API_HOST = '206.217.141.184'
const API_PORT = 3093

function apiRequest(method: string, path: string, token?: string, body?: unknown): Promise<unknown> {
  return new Promise((resolve, reject) => {
    const bodyStr = body ? JSON.stringify(body) : ''
    const headers: Record<string, string | number> = {
      'Content-Type': 'application/json',
      'Content-Length': Buffer.byteLength(bodyStr),
    }
    if (token) headers['Authorization'] = `Bearer ${token}`
    const req = http.request({ hostname: API_HOST, port: API_PORT, path, method, headers }, res => {
      let data = ''
      res.on('data', c => (data += c))
      res.on('end', () => { try { resolve(JSON.parse(data)) } catch { resolve(data) } })
    })
    req.on('error', reject)
    if (bodyStr) req.write(bodyStr)
    req.end()
  })
}

export function setupCosmeticsHandlers(ipcMain: IpcMain): void {
  ipcMain.handle('cosmetics:get', (_e, uuid: string) =>
    apiRequest('GET', `/api/cosmetics/${uuid}`)
  )

  ipcMain.handle('cosmetics:update', async (_e, data: { cape_url?: string | null; cape_type?: string; equipped?: string[] }) => {
    const token = await getBejaToken()
    if (!token) return { error: 'not_logged_in' }
    return apiRequest('PUT', '/api/cosmetics/', token, data)
  })

  ipcMain.handle('cosmetics:inventory', async (_e, uuid: string) =>
    apiRequest('GET', `/api/cosmetics/inventory/${uuid}`)
  )

  // TEMP: mock crate data for UI testing — remove when API is ready
  const MOCK_POOL: Record<string, Array<{ id: string; name: string; type: string }>> = {
    leather:   [{ id: 'antlers', name: 'Antlers', type: 'hat' }, { id: 'leather-acc-1', name: '???', type: 'accessory' }],
    iron:      [{ id: 'wings', name: 'Wings', type: 'wings' }, { id: 'iron-acc-1', name: '???', type: 'accessory' }],
    gold:      [{ id: 'gold-acc-1',    name: '???', type: 'accessory' }],
    diamond:   [{ id: 'diamond-acc-1', name: '???', type: 'accessory' }],
    netherite: [{ id: 'netherite-acc-1', name: '???', type: 'accessory' }],
    enchanted: [{ id: 'enchanted-acc-1', name: '???', type: 'accessory' }],
  }
  const MOCK_RARITY_WEIGHTS: [string, number][] = [
    ['leather', 50], ['iron', 26], ['gold', 13], ['diamond', 7], ['netherite', 3], ['enchanted', 1],
  ]
  function mockRarity() {
    const total = MOCK_RARITY_WEIGHTS.reduce((s, [, w]) => s + w, 0)
    let r = Math.random() * total
    for (const [rarity, w] of MOCK_RARITY_WEIGHTS) { r -= w; if (r <= 0) return rarity }
    return 'leather'
  }
  ipcMain.handle('crates:list', async () => [
    { id: 'test-crate', name: 'Test Crate', description: 'UI testing crate' },
  ])
  ipcMain.handle('crates:open', async () => {
    const rarity  = mockRarity()
    const pool    = MOCK_POOL[rarity]
    const cosmetic = pool[Math.floor(Math.random() * pool.length)]
    return { cosmetic: { ...cosmetic, rarity }, is_new: Math.random() > 0.5 }
  })
  ipcMain.handle('crates:keys', async () => ({ count: 99 }))

  // ── Crafting / Forge ──────────────────────────────────────────────────────
  const CRAFT_CHAIN: Record<string, string> = {
    leather: 'iron', iron: 'gold', gold: 'diamond', diamond: 'netherite',
  }

  ipcMain.handle('crafting:inventory', async () => {
    const token = await getBejaToken()
    if (!token) return []
    return apiRequest('GET', '/api/cosmetics/inventory', token)
  })

  ipcMain.handle('crafting:combine', async (_e, rarity: string) => {
    const token = await getBejaToken()
    if (!token) return { error: 'not_logged_in' }
    const next = CRAFT_CHAIN[rarity]
    if (!next) return { error: 'cannot_craft_enchanted' }
    // TEMP mock — replace with: apiRequest('POST', '/api/crafting/combine', token, { rarity })
    return {
      cosmetic: { id: `forged-${Date.now()}`, name: '???', type: 'accessory', rarity: next },
      is_new: true,
    }
  })
}
