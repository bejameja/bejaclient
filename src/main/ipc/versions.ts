import { IpcMain, BrowserWindow } from 'electron'
import {
  fetchVersionManifest,
  getInstalledVersions,
  installVersion,
  deleteVersion,
  listFabricVersions,
  listForgeVersions,
} from '../services/versionService'

export function setupVersionHandlers(ipcMain: IpcMain): void {
  ipcMain.handle('versions:list-remote', async () => {
    const manifest = await fetchVersionManifest()
    return manifest
  })

  ipcMain.handle('versions:list-installed', () => {
    return getInstalledVersions()
  })

  ipcMain.handle(
    'versions:install',
    async (event, versionId: string, loaderType: string, loaderVersion?: string) => {
      await installVersion(versionId, loaderType, loaderVersion, (task, progress, total) => {
        event.sender.send('versions:progress', { task, progress, total })
      })
      return getInstalledVersions()
    },
  )

  ipcMain.handle('versions:delete', async (_e, versionId: string) => {
    await deleteVersion(versionId)
    return getInstalledVersions()
  })

  ipcMain.handle('versions:list-fabric', async (_e, mcVersion: string) => {
    return await listFabricVersions(mcVersion)
  })

  ipcMain.handle('versions:list-forge', async (_e, mcVersion: string) => {
    return await listForgeVersions(mcVersion)
  })
}
