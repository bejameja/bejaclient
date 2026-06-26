import { ipcMain, app } from 'electron'
import https from 'https'
import http from 'http'
import { createReadStream, createWriteStream, existsSync, mkdirSync, statSync } from 'fs'
import { join } from 'path'

const SCENE_URL = 'https://github.com/bejameja/bejaclient/releases/download/v1.3.3/scene.mp4'

function download(url: string, dest: string): Promise<void> {
  return new Promise((resolve, reject) => {
    const proto = url.startsWith('https') ? https : http
    proto.get(url, (res) => {
      if (res.statusCode === 301 || res.statusCode === 302 || res.statusCode === 303) {
        download(res.headers.location!, dest).then(resolve, reject)
        return
      }
      if (res.statusCode !== 200) { reject(new Error(`HTTP ${res.statusCode}`)); return }
      const file = createWriteStream(dest)
      res.pipe(file)
      file.on('finish', () => file.close(resolve as any))
      file.on('error', reject)
    }).on('error', reject)
  })
}

let videoUrl = ''
let videoServer: http.Server | null = null

function startVideoServer(filePath: string): Promise<string> {
  return new Promise((resolve, reject) => {
    if (videoServer?.listening) {
      resolve(`http://127.0.0.1:49217/scene.mp4`)
      return
    }

    const server = http.createServer((req, res) => {
      const stat = statSync(filePath)
      const range = req.headers.range
      if (range) {
        const [startStr, endStr] = range.replace(/bytes=/, '').split('-')
        const start = parseInt(startStr, 10)
        const end = endStr ? parseInt(endStr, 10) : stat.size - 1
        res.writeHead(206, {
          'Content-Range': `bytes ${start}-${end}/${stat.size}`,
          'Accept-Ranges': 'bytes',
          'Content-Length': String(end - start + 1),
          'Content-Type': 'video/mp4',
        })
        createReadStream(filePath, { start, end }).pipe(res)
      } else {
        res.writeHead(200, {
          'Content-Length': String(stat.size),
          'Content-Type': 'video/mp4',
          'Accept-Ranges': 'bytes',
        })
        createReadStream(filePath).pipe(res)
      }
    })

    server.on('error', (err: NodeJS.ErrnoException) => {
      if (err.code === 'EADDRINUSE') {
        // Port already serving from a prior process — reuse it
        videoServer = null
        resolve(`http://127.0.0.1:49217/scene.mp4`)
      } else {
        reject(err)
      }
    })

    server.listen(49217, '127.0.0.1', () => {
      videoServer = server
      resolve(`http://127.0.0.1:49217/scene.mp4`)
    })

    app.on('before-quit', () => { server.close() })
  })
}

export function setupVideoHandlers() {
  ipcMain.handle('video:get-scene', async () => {
    if (videoUrl) return videoUrl
    const dir = join(app.getPath('userData'), 'assets')
    const dest = join(dir, 'scene.mp4')
    if (!existsSync(dest)) {
      mkdirSync(dir, { recursive: true })
      await download(SCENE_URL, dest)
    }
    videoUrl = await startVideoServer(dest)
    return videoUrl
  })
}
