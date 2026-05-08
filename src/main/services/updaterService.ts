import { autoUpdater, UpdateInfo, ProgressInfo } from 'electron-updater'
import { BrowserWindow, app } from 'electron'
import { getSettings } from './settingsService'

type LogFn = (level: 'INFO' | 'WARN' | 'ERROR', msg: string) => void

export function initAutoUpdater(getWindow: () => BrowserWindow | null, log: LogFn) {
  // Only run in packaged builds — dev has no release channel to check.
  if (!app.isPackaged) return

  autoUpdater.autoDownload = true
  autoUpdater.autoInstallOnAppQuit = true
  autoUpdater.allowPrerelease = false  // client-v* releases are pre-releases; ignore them
  autoUpdater.logger = null

  autoUpdater.on('checking-for-update', () => {
    log('INFO', 'updater: checking for update')
    getWindow()?.webContents.send('updater:checking')
  })

  autoUpdater.on('update-available', (info: UpdateInfo) => {
    log('INFO', `updater: update available — ${info.version}`)
    getWindow()?.webContents.send('updater:available', { version: info.version, releaseNotes: info.releaseNotes })
  })

  autoUpdater.on('update-not-available', () => {
    log('INFO', 'updater: no update available')
    getWindow()?.webContents.send('updater:not-available')
  })

  autoUpdater.on('error', (err: Error) => {
    const msg = err?.message ?? String(err)
    // 404 means no launcher release with latest.yml exists yet — not a real error.
    if (msg.includes('404') || msg.includes('latest.yml')) {
      log('INFO', 'updater: no launcher release found (404) — skipping')
      return
    }
    log('ERROR', `updater: ${msg}`)
    getWindow()?.webContents.send('updater:error', msg)
  })

  autoUpdater.on('download-progress', (progress: ProgressInfo) => {
    getWindow()?.webContents.send('updater:progress', {
      percent: Math.round(progress.percent),
      transferred: progress.transferred,
      total: progress.total,
      bytesPerSecond: progress.bytesPerSecond,
    })
  })

  autoUpdater.on('update-downloaded', (info: UpdateInfo) => {
    log('INFO', `updater: update downloaded — ${info.version}`)
    getWindow()?.webContents.send('updater:downloaded', { version: info.version })
  })

  // Only auto-check on startup when the user has auto-update enabled (default: true).
  const { launcher } = getSettings()
  if (launcher.autoUpdate) {
    setTimeout(() => {
      autoUpdater.checkForUpdatesAndNotify().catch((err: Error) => {
        log('WARN', `updater: auto-check failed — ${err?.message ?? err}`)
      })
    }, 5000)
  } else {
    log('INFO', 'updater: auto-check skipped (disabled in settings)')
  }

  return {
    checkForUpdates: () => autoUpdater.checkForUpdates(),
    downloadUpdate: () => autoUpdater.downloadUpdate(),
    quitAndInstall: () => autoUpdater.quitAndInstall(true, true),
  }
}
