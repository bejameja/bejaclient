import { IpcMain, BrowserWindow } from 'electron'
import { initAutoUpdater } from '../services/updaterService'

type LogFn = (level: 'INFO' | 'WARN' | 'ERROR', msg: string) => void

export function setupUpdaterHandlers(
  ipcMain: IpcMain,
  getWindow: () => BrowserWindow | null,
  log: LogFn,
): void {
  const updater = initAutoUpdater(getWindow, log)

  ipcMain.handle('updater:check', () => updater?.checkForUpdates())
  ipcMain.handle('updater:download', () => updater?.downloadUpdate())
  ipcMain.handle('updater:install', () => {
    updater?.quitAndInstall()
  })
}
