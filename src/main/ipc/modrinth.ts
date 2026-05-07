import { IpcMain, BrowserWindow } from 'electron'
import { searchModrinth, getProjectVersions, downloadMod, downloadModVersion, downloadResourcePack, downloadShader, installModpack, ModrinthProjectType } from '../services/modrinthService'
import { deleteMod, listMods } from '../services/modService'

export function setupModrinthHandlers(ipcMain: IpcMain, mainWindow: () => BrowserWindow | null): void {
  ipcMain.handle('modrinth:search', async (_e, query: string, type: ModrinthProjectType, gameVersion?: string, loader?: string, offset?: number) => {
    try {
      const result = await searchModrinth(query, type, gameVersion, loader, offset)
      console.log(`[modrinth:search] type=${type} hits=${result.hits.length} total=${result.total_hits}`)
      return result
    } catch (e) {
      console.error('[modrinth:search] failed:', e)
      throw e
    }
  })

  ipcMain.handle('modrinth:versions', async (_e, projectId: string, gameVersion?: string, loader?: string) => {
    return getProjectVersions(projectId, gameVersion, loader)
  })

  ipcMain.handle('modrinth:install-mod', async (event, projectId: string, profileId: string) => {
    await downloadMod(projectId, profileId, msg => {
      event.sender.send('modrinth:progress', msg)
    })
    return true
  })

  ipcMain.handle('modrinth:install-modpack', async (event, projectId: string, versionId: string | null) => {
    const result = await installModpack(projectId, versionId, msg => {
      event.sender.send('modrinth:progress', msg)
    })
    return result
  })

  ipcMain.handle('modrinth:install-resourcepack', async (event, projectId: string, profileId: string) => {
    await downloadResourcePack(projectId, profileId, msg => {
      event.sender.send('modrinth:progress', msg)
    })
    return true
  })

  ipcMain.handle('modrinth:install-shader', async (event, projectId: string, profileId: string) => {
    await downloadShader(projectId, profileId, msg => {
      event.sender.send('modrinth:progress', msg)
    })
    return true
  })

  ipcMain.handle('modrinth:swap-mod', async (event, profileId: string, oldModId: string, projectId: string, versionId: string) => {
    deleteMod(profileId, oldModId)
    await downloadModVersion(projectId, versionId, profileId, msg => {
      event.sender.send('modrinth:progress', msg)
    })
    return listMods(profileId)
  })
}
