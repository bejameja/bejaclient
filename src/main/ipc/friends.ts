import { IpcMain, BrowserWindow } from 'electron'
import https from 'https'
import http from 'http'
import { getSelectedAccount } from '../services/authService'
import { connectBejaSocket, disconnectBejaSocket } from '../services/bejaSocketService'

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
    if (bodyStr) req.write(bodyStr)
    req.end()
  })
}

function mojangLookup(username: string): Promise<{ id: string; name: string } | null> {
  return new Promise(resolve => {
    https.get(
      `https://api.mojang.com/users/profiles/minecraft/${encodeURIComponent(username)}`,
      res => {
        if (res.statusCode === 204 || res.statusCode === 404) return resolve(null)
        let data = ''
        res.on('data', c => (data += c))
        res.on('end', () => { try { resolve(JSON.parse(data)) } catch { resolve(null) } })
      },
    ).on('error', () => resolve(null))
  })
}

function addUuidDashes(id: string): string {
  return `${id.slice(0, 8)}-${id.slice(8, 12)}-${id.slice(12, 16)}-${id.slice(16, 20)}-${id.slice(20)}`
}

export function setupFriendsHandlers(ipcMain: IpcMain, getWindow: () => BrowserWindow | null): void {
  ipcMain.handle('friends:connect', () => {
    const account = getSelectedAccount()
    if (!account?.bejaToken) return false
    connectBejaSocket(account.bejaToken, getWindow)
    return true
  })

  ipcMain.handle('friends:disconnect', () => {
    disconnectBejaSocket()
  })

  ipcMain.handle('friends:list', async () => {
    const account = getSelectedAccount()
    if (!account?.bejaToken) return []
    return apiRequest('GET', `/api/friends/${account.uuid}`, account.bejaToken)
  })

  ipcMain.handle('friends:request', async (_e, username: string) => {
    const account = getSelectedAccount()
    if (!account?.bejaToken) return { error: 'not_logged_in' }
    const profile = await mojangLookup(username)
    if (!profile) return { error: 'not_found' }
    const targetUuid = addUuidDashes(profile.id)
    return apiRequest('POST', '/api/friends/request', account.bejaToken, { targetUuid })
  })

  ipcMain.handle('friends:accept', async (_e, requesterUuid: string) => {
    const account = getSelectedAccount()
    if (!account?.bejaToken) return { error: 'not_logged_in' }
    return apiRequest('POST', '/api/friends/accept', account.bejaToken, { requesterUuid })
  })

  ipcMain.handle('friends:remove', async (_e, friendUuid: string) => {
    const account = getSelectedAccount()
    if (!account?.bejaToken) return { error: 'not_logged_in' }
    return apiRequest('DELETE', `/api/friends/${friendUuid}`, account.bejaToken)
  })
}
