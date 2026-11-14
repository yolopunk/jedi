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
  action: 'copy' | 'save' | 'chat' | 'ocr' | 'pin' | 'scroll'
  savedPath: string | null
  /** 警告码（如 stitch_fallback），由前端映射 i18n 文案 */
  warnings: string[]
}

export interface ScreenshotItem {
  name: string
  path: string
  size: number
  modifiedMs: number
  /** 条目类型：image | video */
  kind: 'image' | 'video'
}

export interface ScrollStatus {
  frameCount: number
  /** 已捕获帧的缩略图（data URL 序列），NEG 胶片条用 */
  thumbs: string[]
}

export interface ToolbarInfo {
  mode: 'scroll' | 'record'
  frameCount: number
  /** 录屏最长时长（毫秒），滚动模式为 0 */
  maxDurationMs: number
}

export interface RecordStartInfo {
  width: number
  height: number
  fps: number
  maxDurationMs: number
}

export interface RecordOutcome {
  path: string
}

type PhysicalRect = { x: number; y: number; width: number; height: number }

/** 触发一轮截图：后端捕获显示器并弹出覆盖窗口 */
export function screenshotBegin(): Promise<MonitorFrameInfo[]> {
  return invoke('screenshot_begin')
}

/** 覆盖窗口挂载时查询本显示器的冻结帧信息 */
export function screenshotFrameInfo(monitorIndex: number): Promise<MonitorFrameInfo> {
  return invoke('screenshot_frame_info', { monitorIndex })
}

/** 确认选区并执行输出动作（copy / save / chat / ocr / pin），坐标为物理像素 */
export function screenshotFinish(
  monitorIndex: number,
  rect: { x: number; y: number; width: number; height: number },
  action: 'copy' | 'save' | 'chat' | 'ocr' | 'pin'
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

/** 进入滚动截取模式：固定选区并捕获第一屏（坐标为物理像素） */
export function screenshotScrollBegin(
  monitorIndex: number,
  rect: PhysicalRect
): Promise<ScrollStatus> {
  return invoke('screenshot_scroll_begin', { monitorIndex, ...rect })
}

/** 滚动截取：捕获一屏追加进帧序列 */
export function screenshotScrollCapture(): Promise<ScrollStatus> {
  return invoke('screenshot_scroll_capture')
}

/** 完成滚动截取：重叠检测拼接为长图并入库 */
export function screenshotScrollFinish(): Promise<ScreenshotOutcome> {
  return invoke('screenshot_scroll_finish')
}

/** 取消滚动截取，丢弃全部帧 */
export function screenshotScrollCancel(): Promise<void> {
  return invoke('screenshot_scroll_cancel')
}

/** 注册（换绑）截图全局快捷键，accelerator 形如 "F1" / "Ctrl+Alt+S" */
export function setCaptureHotkey(accelerator: string): Promise<void> {
  return invoke('set_capture_hotkey', { accelerator })
}

/** 读取当前生效的截图快捷键 */
export function getCaptureHotkey(): Promise<string> {
  return invoke('get_capture_hotkey')
}

/** 删除截图库中的一条记录（仅限截图目录内） */
export function deleteScreenshot(path: string): Promise<void> {
  return invoke('screenshot_delete', { path })
}

/** 悬浮控制条查询当前模式（scroll / record）与状态 */
export function screenshotToolbarInfo(): Promise<ToolbarInfo> {
  return invoke('screenshot_toolbar_info')
}

/** 开始选区录屏（fps 默认 15），返回实际尺寸与最长时长 */
export function startRegionRecord(
  monitorIndex: number,
  rect: PhysicalRect,
  fps?: number
): Promise<RecordStartInfo> {
  return invoke('start_region_record', { monitorIndex, ...rect, fps })
}

/** 停止录屏并等待封装完成，返回 MP4 路径 */
export function stopRegionRecord(): Promise<RecordOutcome> {
  return invoke('stop_region_record')
}

/** 用系统默认程序打开截图库文件（录屏 MP4 用） */
export function openScreenshotFile(path: string): Promise<void> {
  return invoke('screenshot_open_file', { path })
}

export function recognizeScreenshot(
  path: string
): Promise<{ text: string; language: string; engine: string }> {
  return invoke('screenshot_ocr', { path })
}

export function copyScreenshotText(text: string): Promise<void> {
  return invoke('screenshot_copy_text', { text })
}

/** 弹出系统文件选择器，把选中的图片导入截图库；取消时返回空串 */
export function importScreenshot(): Promise<string> {
  return invoke('screenshot_import')
}

/** 把截图库中的图片复制到系统剪贴板 */
export function copyScreenshotFile(path: string): Promise<void> {
  return invoke('screenshot_copy_file', { path })
}

/** 把一张图片贴到屏幕上：置顶悬浮窗，可拖动 / 缩放 / Esc 收起 */
export function pinScreenshotFile(path: string): Promise<void> {
  return invoke('screenshot_pin_file', { path })
}

/** 贴图窗口挂载时领取自己的图片路径 */
export function takePinPath(): Promise<string> {
  return invoke('screenshot_pin_take')
}
