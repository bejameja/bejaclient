import { IpcMain, dialog } from 'electron'
import { listMods, toggleMod, installMod, deleteMod, openModsFolder } from '../services/modService'

export function setupModHandlers(ipcMain: IpcMain): void {
  ipcMain.handle('mods:list', (_e, profileId: string) => {
    return listMods(profileId)
  })

  ipcMain.handle('mods:install', async (_e, profileId: string, filePath?: string) => {
    if (!filePath) {
      const result = await dialog.showOpenDialog({
        title: 'Select Mod File',
        filters: [{ name: 'Minecraft Mods', extensions: ['jar'] }],
        properties: ['openFile', 'multiSelections'],
      })
      if (result.canceled || !result.filePaths.length) return listMods(profileId)
      for (const fp of result.filePaths) {
        installMod(profileId, fp)
      }
      return listMods(profileId)
    }
    return installMod(profileId, filePath)
  })

  ipcMain.handle('mods:toggle', (_e, profileId: string, modId: string) => {
    return toggleMod(profileId, modId)
  })

  ipcMain.handle('mods:delete', (_e, profileId: string, modId: string) => {
    return deleteMod(profileId, modId)
  })

  ipcMain.handle('mods:open-folder', (_e, profileId: string) => {
    openModsFolder(profileId)
  })
}
