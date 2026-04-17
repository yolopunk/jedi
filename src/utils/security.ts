/**
 * 安全工具模块
 * Phase 1: 输入验证和输出编码
 */

import DOMPurify from 'dompurify'
import MarkdownIt from 'markdown-it'

// ========== 输入验证 ==========

export interface ValidationResult {
  valid: boolean
  sanitized: string
  warnings: string[]
}

export interface ValidationOptions {
  maxLength?: number
  allowUrls?: boolean
  allowMarkdown?: boolean
  blocklist?: string[]
}

const DEFAULT_OPTIONS: ValidationOptions = {
  maxLength: 10000,
  allowUrls: true,
  allowMarkdown: true,
  blocklist: [
    'javascript:',
    'data:',
    'vbscript:',
    'livescript:',
    'mocha:',
    'eval(',
    'expression(',
    'document.cookie',
    'localStorage',
    'sessionStorage',
  ],
}

// 危险的 HTML 标签和属性
const DANGEROUS_TAGS = [
  'script',
  'iframe',
  'form',
  'input',
  'button',
  'textarea',
  'select',
  'option',
]
const DANGEROUS_ATTRS = [
  'onload',
  'onerror',
  'onclick',
  'onmouseover',
  'onfocus',
  'onblur',
  'onsubmit',
]

/**
 * 验证和清理用户输入
 * @param input 原始用户输入
 * @param options 验证选项
 * @returns 验证结果
 */
export function validateUserInput(input: string, options?: ValidationOptions): ValidationResult {
  const opts = { ...DEFAULT_OPTIONS, ...options }
  const warnings: string[] = []
  let sanitized = input

  // 1. 基本空值检查
  if (!input || input.trim().length === 0) {
    return { valid: true, sanitized: '', warnings: [] }
  }

  // 2. 长度检查
  if (opts.maxLength && input.length > opts.maxLength) {
    warnings.push(`输入长度超过限制（最大 ${opts.maxLength} 字符）`)
    sanitized = sanitized.substring(0, opts.maxLength)
  }

  // 3. 检查 blocklist
  if (opts.blocklist) {
    for (const pattern of opts.blocklist) {
      if (sanitized.toLowerCase().includes(pattern.toLowerCase())) {
        warnings.push(`检测到潜在危险内容: ${pattern}`)
        const regex = new RegExp(pattern, 'gi')
        sanitized = sanitized.replace(regex, '[REDACTED]')
      }
    }
  }

  // 4. 移除危险标签（基本清理，作为 DOMPurify 的补充）
  for (const tag of DANGEROUS_TAGS) {
    const regex = new RegExp(`<${tag}[^>]*>.*?</${tag}>`, 'gi')
    if (regex.test(sanitized)) {
      warnings.push(`检测到危险标签: <${tag}>`)
    }
  }

  // 5. 移除危险属性
  for (const attr of DANGEROUS_ATTRS) {
    const regex = new RegExp(`${attr}\\s*=`, 'gi')
    if (regex.test(sanitized)) {
      warnings.push(`检测到危险属性: ${attr}`)
    }
  }

  // 6. 验证 URL 格式（如果允许 URL）
  if (opts.allowUrls) {
    const urlRegex = /(https?:\/\/[^\s]+)/gi
    const urls = sanitized.match(urlRegex) || []
    for (const url of urls) {
      try {
        new URL(url)
      } catch {
        warnings.push(`检测到无效 URL: ${url.substring(0, 50)}...`)
      }
    }
  }

  return {
    valid: warnings.length === 0,
    sanitized,
    warnings,
  }
}

/**
 * 验证 API Key 格式
 * @param key API Key
 * @param provider 提供商名称
 * @returns 验证结果
 */
export function validateApiKey(key: string, provider?: string): ValidationResult {
  const warnings: string[] = []

  if (!key || key.trim().length === 0) {
    return { valid: false, sanitized: '', warnings: ['API Key 不能为空'] }
  }

  const trimmedKey = key.trim()

  // 通用检查
  if (trimmedKey.length < 10) {
    warnings.push('API Key 长度看起来太短')
  }

  // 提供商特定的格式检查
  if (provider) {
    switch (provider.toLowerCase()) {
      case 'openai':
        if (!trimmedKey.startsWith('sk-')) {
          warnings.push('OpenAI API Key 应该以 "sk-" 开头')
        }
        if (trimmedKey.length < 50) {
          warnings.push('OpenAI API Key 长度看起来不正确')
        }
        break
      case 'anthropic':
        if (!trimmedKey.startsWith('sk-ant-')) {
          warnings.push('Anthropic API Key 应该以 "sk-ant-" 开头')
        }
        break
    }
  }

  return {
    valid: warnings.length === 0,
    sanitized: trimmedKey,
    warnings,
  }
}

/**
 * 验证提供商端点 URL
 * @param endpoint 端点 URL
 * @returns 验证结果
 */
export function validateEndpoint(endpoint: string): ValidationResult {
  const warnings: string[] = []

  if (!endpoint || endpoint.trim().length === 0) {
    return { valid: true, sanitized: '', warnings: [] }
  }

  const trimmedEndpoint = endpoint.trim()

  try {
    const url = new URL(trimmedEndpoint)

    // 只允许 http 和 https
    if (url.protocol !== 'http:' && url.protocol !== 'https:') {
      warnings.push('只允许 HTTP 和 HTTPS 协议')
    }

    // 阻止访问本地网络（除了明确允许的 localhost）
    const hostname = url.hostname.toLowerCase()
    const localPatterns = ['127.0.0.1', 'localhost', '::1', '0.0.0.0']

    // 允许 localhost 和 127.0.0.1（用于 Ollama 等本地服务）
    const isLocal = localPatterns.some(
      pattern => hostname === pattern || hostname.endsWith('.' + pattern)
    )
    const isPrivateNetwork = hostname.match(/^(10\.|172\.(1[6-9]|2[0-9]|3[01])\.|192\.168\.)/)

    if (isPrivateNetwork && !isLocal) {
      warnings.push('不建议使用私有网络地址')
    }
  } catch {
    warnings.push('无效的 URL 格式')
  }

  return {
    valid: warnings.length === 0,
    sanitized: trimmedEndpoint,
    warnings,
  }
}

// ========== 输出编码 ==========

// 配置 MarkdownIt（禁用 HTML）
const md = new MarkdownIt({
  html: false, // 禁用 HTML 标签
  xhtmlOut: false,
  breaks: true,
  linkify: true,
  typographer: true,
  quotes: '""\'\'',
})

// 配置 linkify 只允许 http/https
md.linkify.set({
  fuzzyLink: false,
  fuzzyEmail: true,
})

/**
 * 安全渲染 Markdown
 * @param markdown Markdown 文本
 * @returns 安全的 HTML
 */
export function renderMarkdownSafe(markdown: string): string {
  if (!markdown) {
    return ''
  }

  // 1. 先通过 MarkdownIt 渲染（已禁用 HTML）
  const rendered = md.render(markdown)

  // 2. 再通过 DOMPurify 进行最终清理
  const clean = DOMPurify.sanitize(rendered, {
    ALLOWED_TAGS: [
      'p',
      'br',
      'strong',
      'em',
      'u',
      's',
      'code',
      'pre',
      'h1',
      'h2',
      'h3',
      'h4',
      'h5',
      'h6',
      'ul',
      'ol',
      'li',
      'blockquote',
      'a',
      'table',
      'thead',
      'tbody',
      'tr',
      'th',
      'td',
      'hr',
    ],
    ALLOWED_ATTR: ['href', 'title', 'class'],
    ALLOW_DATA_ATTR: false,
    ALLOW_UNKNOWN_PROTOCOLS: false,
    ALLOWED_URI_REGEXP: /^(?:(?:https?|mailto|tel):|#)/i,
  })

  return clean
}

/**
 * HTML 实体编码
 * @param text 原始文本
 * @returns 编码后的文本
 */
export function encodeHtml(text: string): string {
  if (!text) {
    return ''
  }

  const div = document.createElement('div')
  div.textContent = text
  return div.innerHTML
}

/**
 * 移除所有 HTML 标签
 * @param html HTML 文本
 * @returns 纯文本
 */
export function stripHtml(html: string): string {
  if (!html) {
    return ''
  }

  const div = document.createElement('div')
  div.innerHTML = html
  return div.textContent || div.innerText || ''
}

// ========== 消息验证 ==========

export interface ChatMessageValidation {
  valid: boolean
  content: string
  warnings: string[]
  metadata: {
    wordCount: number
    charCount: number
    hasUrls: boolean
    hasCode: boolean
  }
}

/**
 * 验证聊天消息
 * @param content 消息内容
 * @param isUser 是否是用户消息
 * @returns 验证结果
 */
export function validateChatMessage(
  content: string,
  isUser: boolean = true
): ChatMessageValidation {
  const warnings: string[] = []
  let sanitizedContent = content

  // 用户消息需要更严格的验证
  if (isUser) {
    const validation = validateUserInput(content, {
      maxLength: 50000,
      allowUrls: true,
      allowMarkdown: true,
    })
    sanitizedContent = validation.sanitized
    warnings.push(...validation.warnings)
  }

  // 分析内容元数据
  const words = sanitizedContent
    .trim()
    .split(/\s+/)
    .filter(w => w.length > 0)
  const urlRegex = /https?:\/\/[^\s]+/gi
  const codeRegex = /```[\s\S]*?```|`[^`]+`/g

  const metadata = {
    wordCount: words.length,
    charCount: sanitizedContent.length,
    hasUrls: urlRegex.test(sanitizedContent),
    hasCode: codeRegex.test(sanitizedContent),
  }

  // 额外检查
  if (metadata.charCount === 0) {
    warnings.push('消息内容不能为空')
  }

  if (isUser && metadata.charCount > 10000) {
    warnings.push('消息较长，可能会影响响应速度')
  }

  return {
    valid: warnings.length === 0 || (warnings.length === 1 && metadata.charCount > 10000),
    content: sanitizedContent,
    warnings,
    metadata,
  }
}

// ========== 导出 ==========

export default {
  validateUserInput,
  validateApiKey,
  validateEndpoint,
  renderMarkdownSafe,
  encodeHtml,
  stripHtml,
  validateChatMessage,
}
