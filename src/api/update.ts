import { relaunch } from '@tauri-apps/plugin-process'
import { check, type Update } from '@tauri-apps/plugin-updater'

export interface UpdateInfo {
  version: string
  date: string
  body: string
}

let currentUpdate: Update | null = null

// Check for available updates using tauri-plugin-updater
export async function checkUpdate(): Promise<UpdateInfo | null> {
  try {
    const update = await check()
    if (update?.available) {
      currentUpdate = update
      return {
        version: update.version,
        date: update.date || new Date().toISOString(),
        body: update.body || '',
      }
    }
    return null
  } catch (error) {
    console.error('Failed to check for updates:', error)
    return null
  }
}

// Download the update
export async function downloadUpdate(
  onProgress: (total: number, current: number) => void
): Promise<void> {
  if (!currentUpdate) {
    throw new Error('No update available. Call checkUpdate first.')
  }

  try {
    let downloaded = 0
    let contentLength = 0

    await currentUpdate.download(event => {
      switch (event.event) {
        case 'Started':
          contentLength = event.data.contentLength || 0
          break
        case 'Progress':
          downloaded += event.data.chunkLength
          if (contentLength > 0) {
            onProgress(contentLength, downloaded)
          }
          break
      }
    })
  } catch (error) {
    console.error('Failed to download update:', error)
    throw error
  }
}

// Install the update
export async function installUpdate(): Promise<void> {
  if (!currentUpdate) {
    throw new Error('No update available')
  }

  try {
    await currentUpdate.install()
    await relaunch()
  } catch (error) {
    console.error('Failed to install update:', error)
    throw error
  }
}

// Download and install in one step
export async function downloadAndInstallUpdate(
  onProgress: (total: number, current: number) => void
): Promise<void> {
  if (!currentUpdate) {
    // Try to check again if currentUpdate is missing
    const info = await checkUpdate()
    if (!info || !currentUpdate) {
      throw new Error('No update available')
    }
  }

  try {
    let downloaded = 0
    let contentLength = 0

    await currentUpdate.downloadAndInstall(event => {
      switch (event.event) {
        case 'Started':
          contentLength = event.data.contentLength || 0
          break
        case 'Progress':
          downloaded += event.data.chunkLength
          if (contentLength > 0) {
            onProgress(contentLength, downloaded)
          }
          break
        case 'Finished':
          break
      }
    })

    await relaunch()
  } catch (error) {
    console.error('Failed to download and install update:', error)
    throw error
  }
}
