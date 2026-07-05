import DOMPurify from 'dompurify'
import hljs from 'highlight.js'
import MarkdownIt from 'markdown-it'

export function createMarkdownRenderer(): MarkdownIt {
  return new MarkdownIt({
    html: false,
    linkify: true,
    typographer: true,
    highlight(str: string, lang: string): string {
      if (lang && hljs.getLanguage(lang)) {
        try {
          const safeLang = lang.replace(/[^\w-]/g, '')
          return `<pre class="hljs" data-lang="${safeLang}"><code>${hljs.highlight(str, { language: lang }).value}</code></pre>`
        } catch (_) {}
      }
      return `<pre class="hljs"><code>${new MarkdownIt().utils.escapeHtml(str)}</code></pre>`
    },
  })
}

export function renderSafe(md: MarkdownIt, content: string): string {
  if (!content) return ''
  const rendered = md.render(content)
  return DOMPurify.sanitize(rendered, {
    ADD_ATTR: ['class'],
    ADD_TAGS: ['pre', 'code'],
  })
}

export const sharedMd = createMarkdownRenderer()
