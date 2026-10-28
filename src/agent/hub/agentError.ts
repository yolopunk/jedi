// Failure classification for external agent runs. Every way a turn can end
// badly maps to one `AgentErrorKind` with a user-facing reason + hint; the raw
// stderr tail / exit code go into `detail` for the expandable technical area.
// Output is scanned for patterns only — it is never parsed for secrets, and
// the tail is capped so a chatty agent cannot bloat persisted metadata.

export type AgentErrorKind =
  | 'not_found'
  | 'auth'
  | 'model'
  | 'network'
  | 'quota'
  | 'refused'
  | 'crash'
  | 'timeout'

export class AgentRunError extends Error {
  constructor(
    public kind: AgentErrorKind,
    message: string,
    public hint?: string,
    public detail?: string
  ) {
    super(message)
    this.name = 'AgentRunError'
  }
}

interface Rule {
  kind: AgentErrorKind
  test: RegExp
  /** Interactive login prompt: the process will hang, so fail immediately. */
  early?: boolean
}

// Order matters: the first matching rule wins.
const RULES: Rule[] = [
  { kind: 'auth', early: true, test: /waiting for (authentication|auth)|open (this|the following) (url|link)|accounts\.google\.com\/o\/oauth|device[_ ]code/i },
  { kind: 'auth', early: true, test: /no api key found|no auth type is selected|please set an auth method|please (log ?in|sign ?in|authenticate)|run `?\S+ (auth )?login`?/i },
  { kind: 'auth', test: /\b401\b|unauthori[sz]ed|invalid[_ ]api[_ ]key|authentication (failed|required|error)|not (logged|signed) in|token (has )?expired|oauth token|credentials? (missing|invalid|expired)/i },
  { kind: 'quota', test: /\b429\b|rate[_ ]limit|quota|insufficient[_ ]credit|usage limit|billing|too many requests|resource_exhausted/i },
  { kind: 'model', test: /model[^\n]{0,40}(not found|not exist|unavailable|not supported|invalid)|unknown model|no model (configured|selected|specified)|model_not_found/i },
  { kind: 'network', test: /ENOTFOUND|ECONNREFUSED|ECONNRESET|ETIMEDOUT|EAI_AGAIN|getaddrinfo|network (error|is unreachable)|socket hang up|fetch failed|connection (refused|reset|timed out)|proxy/i },
  { kind: 'refused', test: /\b403\b|forbidden|permission denied|not allowed|refus|overloaded|\b5\d\d\b (internal|bad gateway|service unavailable)/i },
  { kind: 'not_found', test: /command not found|no such file or directory|program not found|is not recognized as an internal|ENOENT|cannot find (module|the file)/i },
]

const COPY: Record<AgentErrorKind, { reason: string; hint: string }> = {
  not_found: {
    reason: '找不到 Agent 程序或启动路径无效',
    hint: '确认该 Agent 已安装且可在终端运行，然后在引擎列表中点击“重新检测”。',
  },
  auth: {
    reason: 'Agent 未登录或登录已失效',
    hint: '在终端中运行该 Agent 完成登录（或配置它自己的 API Key），再回到 Jedi 重试。',
  },
  model: {
    reason: 'Agent 的模型未配置或不可用',
    hint: '检查该 Agent 自身配置中的模型名，修改后点击“重新检测”。',
  },
  network: {
    reason: '网络连接失败',
    hint: '检查网络或代理设置后重试。',
  },
  quota: {
    reason: '额度不足或请求过于频繁',
    hint: '稍后重试，或检查该 Agent 账号的额度与计费。',
  },
  refused: {
    reason: '服务端拒绝了请求',
    hint: '查看技术详情中的原始报错；可能是权限、服务过载或策略限制。',
  },
  crash: {
    reason: 'Agent 进程异常退出',
    hint: '查看技术详情中的输出；可在终端直接运行该 Agent 复现。',
  },
  timeout: {
    reason: 'Agent 长时间没有响应',
    hint: '已自动停止本轮。可能在等待交互式确认或登录，请在终端运行该 Agent 检查。',
  },
}

export function agentError(
  kind: AgentErrorKind,
  agentName: string,
  detail?: string,
  reasonOverride?: string
): AgentRunError {
  const copy = COPY[kind]
  return new AgentRunError(kind, `${agentName}：${reasonOverride ?? copy.reason}`, copy.hint, detail)
}

function classifyText(text: string): Rule | undefined {
  return RULES.find(rule => rule.test.test(text))
}

const TAIL_LINES = 40
const LINE_MAX = 400

/**
 * Per-run diagnostics: collects the stderr / non-protocol stdout tail, spots
 * interactive login prompts early, and turns exits / spawn failures into a
 * classified `AgentRunError`.
 */
export function createAgentDiagnostics(agentName: string) {
  const tail: string[] = []

  const push = (line: string) => {
    const trimmed = line.trimEnd()
    if (!trimmed) return
    tail.push(trimmed.length > LINE_MAX ? `${trimmed.slice(0, LINE_MAX)}…` : trimmed)
    if (tail.length > TAIL_LINES) tail.shift()
  }

  const detail = (extra?: string) => [extra, ...tail].filter(Boolean).join('\n')

  return {
    /** Record an output line; returns an error when the agent is blocked on a login prompt. */
    line(line: string): AgentRunError | null {
      push(line)
      const rule = classifyText(line)
      return rule?.early ? agentError(rule.kind, agentName, detail()) : null
    },
    /** Classified failure for a non-zero exit, or `fallback` text when nothing matched. */
    exit(code: number | null | undefined, protocolError?: string | null): AgentRunError {
      const header = `exit code: ${code ?? 'unknown'}`
      const text = [protocolError, ...tail].filter(Boolean).join('\n')
      const rule = classifyText(text)
      if (rule) return agentError(rule.kind, agentName, detail(header))
      // Shell "command not found" exit codes.
      if (code === 127 || code === 9009) return agentError('not_found', agentName, detail(header))
      return agentError('crash', agentName, detail(protocolError ? `${header}\n${protocolError}` : header))
    },
    /** Error reported inside the agent's own protocol stream (e.g. result.is_error). */
    protocol(message: string): AgentRunError {
      const rule = classifyText(message)
      return agentError(rule?.kind ?? 'refused', agentName, detail(message), rule ? undefined : message)
    },
    spawn(e: unknown): AgentRunError {
      const msg = String(e)
      const rule = classifyText(msg)
      return agentError(rule?.kind ?? 'not_found', agentName, detail(msg))
    },
    detail,
  }
}

export type AgentDiagnostics = ReturnType<typeof createAgentDiagnostics>
