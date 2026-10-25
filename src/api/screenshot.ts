/**
 * 截图裁剪 API
 * 覆盖窗口框选后按物理像素选区回传，由后端裁剪输出
 */
import { invoke } from '@tauri-apps/api/core'

export interface MonitorFrameInfo {
  index: number
  framePath: string
  x: number
  y: number
  width: number
  height: number
}

export interface ScreenshotOutcome {
  action: 'copy' | 'save' | 'chat' | 'ocr'
  savedPath: string | null
}

export interface ScreenshotItem {
  name: string
  path: string
  size: number
  modifiedMs: number
}

/** 触发一轮截图：后端捕获显示器并弹出覆盖窗口 */
export function screenshotBegin(): Promise<MonitorFrameInfo[]> {
  return invoke('screenshot_begin')
}

/** 覆盖窗口挂载时查询本显示器的冻结帧信息 */
export function screenshotFrameInfo(monitorIndex: number): Promise<MonitorFrameInfo> {
  return invoke('screenshot_frame_info', { monitorIndex })
}

/** 确认选区并执行输出动作（copy / save / chat），坐标为物理像素 */
export function screenshotFinish(
  monitorIndex: number,
  rect: { x: number; y: number; width: number; height: number },
  action: 'copy' | 'save' | 'chat' | 'ocr'
): Promise<ScreenshotOutcome> {
  return invoke('screenshot_finish', { monitorIndex, ...rect, action })
}

/** 取消本轮截图 */
export function screenshotCancel(): Promise<void> {
  return invoke('screenshot_cancel')
}

/** 历史截图列表（按时间倒序，最多 24 张） */
export function listScreenshots(): Promise<ScreenshotItem[]> {
  return invoke('list_screenshots')
}

/** 读取图片文件为 data URL（AI 聊天多模态消息用） */
export function readImageDataUrl(path: string): Promise<string> {
  return invoke('read_image_data_url', { path })
}

export function saveStyledScreenshot(dataUrl: string): Promise<string> {
  return invoke('screenshot_save_styled', { dataUrl })
}

export function recognizeScreenshot(
  path: string
): Promise<{ text: string; language: string; engine: string }> {
  return invoke('screenshot_ocr', { path })
}

export function copyScreenshotText(text: string): Promise<void> {
  return invoke('screenshot_copy_text', { text })
}
