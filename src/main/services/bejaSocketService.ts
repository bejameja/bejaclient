import { io, Socket } from 'socket.io-client'
import { BrowserWindow } from 'electron'

const BEJA_API = 'http://206.217.141.184:3093'

let socket: Socket | null = null

export function connectBejaSocket(token: string, getWindow: () => BrowserWindow | null): void {
  if (socket) {
    socket.disconnect()
    socket = null
  }

  socket = io(BEJA_API, {
    auth: { token },
    reconnection: true,
    transports: ['websocket'],
  })

  socket.on('friend:online',  data => getWindow()?.webContents.send('friend:online',  data))
  socket.on('friend:offline', data => getWindow()?.webContents.send('friend:offline', data))
  socket.on('friend:request', data => getWindow()?.webContents.send('friend:request', data))
  socket.on('connect_error', err => console.error('[Socket]', err.message))
}

export function disconnectBejaSocket(): void {
  socket?.disconnect()
  socket = null
}
