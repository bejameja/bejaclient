import { app, Tray, Menu, nativeImage, BrowserWindow } from 'electron'
import { join } from 'path'

let tray: Tray | null = null
let getWindow: (() => BrowserWindow | null) | null = null

export function initTrayService(windowGetter: () => BrowserWindow | null): void {
  getWindow = windowGetter
}

function getTrayIcon(): Electron.NativeImage {
  const p = app.isPackaged
    ? join(process.resourcesPath, 'icon.ico')
    : join(__dirname, '../../../resources/icon.ico')
  return nativeImage.createFromPath(p).resize({ width: 16, height: 16 })
}

export function showTray(): void {
  if (tray) return
  tray = new Tray(getTrayIcon())
  tray.setToolTip('BejaClient — Game running')
  tray.setContextMenu(Menu.buildFromTemplate([
    { label: 'Open Launcher', click: () => { const w = getWindow?.(); w?.show(); w?.focus() } },
    { type: 'separator' },
    { label: 'Quit', click: () => app.quit() },
  ]))
  tray.on('double-click', () => { const w = getWindow?.(); w?.show(); w?.focus() })
}

export function hideTray(): void {
  tray?.destroy()
  tray = null
}
