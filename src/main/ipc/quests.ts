import { IpcMain } from 'electron'
import http from 'http'
import { getBejaToken } from '../services/bejaAuth'

const API_HOST = '206.217.141.184'
const API_PORT = 3093

function apiRequest(method: string, path: string, token: string, body?: unknown): Promise<unknown> {
  return new Promise((resolve, reject) => {
    const bodyStr = body ? JSON.stringify(body) : ''
    const req = http.request({
      hostname: API_HOST,
      port: API_PORT,
      path,
      method,
      headers: {
        Authorization: `Bearer ${token}`,
        'Content-Type': 'application/json',
        'Content-Length': Buffer.byteLength(bodyStr),
      },
    }, res => {
      let data = ''
      res.on('data', c => (data += c))
      res.on('end', () => { try { resolve(JSON.parse(data)) } catch { resolve(data) } })
    })
    req.on('error', reject)
    req.setTimeout(8000, () => { req.destroy(); reject(new Error('timeout')) })
    if (bodyStr) req.write(bodyStr)
    req.end()
  })
}

export function setupQuestsHandlers(ipcMain: IpcMain): void {
  ipcMain.handle('quests:list', async () => {
    const token = await getBejaToken()
    if (!token) return { week: '', quests: [] }
    return apiRequest('GET', '/api/quests', token).catch(() => ({ week: '', quests: [] }))
  })

  ipcMain.handle('quests:progress', async (_e, questId: string, amount: number) => {
    const token = await getBejaToken()
    if (!token) return null
    return apiRequest('POST', '/api/quests/progress', token, { questId, amount }).catch(() => null)
  })

  ipcMain.handle('quests:claim', async (_e, questId: string) => {
    const token = await getBejaToken()
    if (!token) return { awarded: false }
    return apiRequest('POST', '/api/quests/claim', token, { questId }).catch(() => ({ awarded: false }))
  })

  ipcMain.handle('quests:leaderboard', async () => {
    const token = await getBejaToken()
    if (!token) return { entries: [], myRank: null }
    return apiRequest('GET', '/api/quests/leaderboard', token).catch(() => ({ entries: [], myRank: null }))
  })
}
