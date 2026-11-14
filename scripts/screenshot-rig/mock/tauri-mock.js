/**
 * Jedi screenshot rig — Tauri IPC shim with deterministic demo data.
 *
 * Injected as a classic <head> script BEFORE the app module graph, so
 * @tauri-apps/api's `window.__TAURI_INTERNALS__.invoke` resolves here.
 * `window.__TAURI__` stays absent on purpose: useStorage() then falls back
 * to localStorage, which this file seeds with a fixed set of preferences.
 *
 * All personal data here is fictional; nothing from the real machine leaks
 * into the README screenshots.
 */
(() => {
  if (window.__TAURI_INTERNALS__) return // idempotent under HMR reloads

  const RIG = '/__rig/assets/'
  const NOW = Date.now()
  const HOUR = 3600 * 1000
  const iso = ms => new Date(ms).toISOString()

  // ---------- seeded preferences (localStorage, JSON-encoded like useStorage) ----------
  const seed = {
    'theme-mode': JSON.stringify('dark'),
    language: JSON.stringify('zh'),
    'jedi-last-model': JSON.stringify({ providerId: 'openai', modelId: 'gpt-5.1' }),
    'agent-hub-engine': JSON.stringify('builtin'),
  }
  for (const [key, value] of Object.entries(seed)) {
    try {
      if (localStorage.getItem(key) === null) localStorage.setItem(key, value)
    } catch (e) {
      /* storage unavailable — keep defaults */
    }
  }
  try {
    localStorage.setItem('jedi-sidebar-width', '220')
  } catch (e) {
    /* ignore */
  }

  // ---------- transformCallback registry (event plugin plumbing) ----------
  let cbSeq = 0
  let evSeq = 100
  const callbacks = new Map()

  // ---------- system info ----------
  const GB = 1024 * 1024 * 1024
  const osInfo = {
    name: 'Windows',
    kernel_version: '10.0.26300',
    os_version: '26300',
    long_os_version: 'Windows 11 Pro',
    host_name: 'DEV-WORKSTATION',
    arch: 'x86_64',
    boot_time: Math.floor((NOW - 27 * HOUR) / 1000),
    load_average: [0.42, 0.38, 0.31],
    memory: { total: 32 * GB, used: 11.4 * GB, available: 20.6 * GB, free: 8.1 * GB },
    cpus: [{ name: 'CPU0', brand: 'AMD Ryzen 9 7950X', vendor: 'AMD', frequency: 4500, usage: 18.4 }],
    gpus: [{ name: 'NVIDIA GeForce RTX 4080', memory_total: 16 * GB, memory_used: 3.2 * GB, memory_free: 12.8 * GB, temperature: '48' }],
    disks: [{ type_: 'SSD', name: 'NVMe 0', file_system: 'NTFS', mount_point: 'C:', total_space: 2 * 1024 * GB, available_space: 812 * GB, is_removable: false }],
    networks: { '以太网': { mac_addr: 'A1:B2:C3:D4:E5:F6', received: 412 * 1024, transmitted: 48 * 1024 } },
    metrics: {
      cpu_usage: 18.4,
      gpu_temp: '48',
      gpu_memory_total: 16 * GB,
      gpu_memory_used: 3.2 * GB,
      memory_total: 32 * GB,
      memory_used: 11.4 * GB,
      disk_total_space: 2 * 1024 * GB,
      disk_used_space: 1.2 * 1024 * GB,
      network_received: 412 * 1024,
      network_transmitted: 48 * 1024,
    },
  }

  // ---------- hosts ----------
  const hostsGroups = [
    {
      name: '测试环境',
      hosts: [
        { ip: '10.50.128.32', domain: 'api.dev.io', disabled: false },
        { ip: '10.50.128.32', domain: 'grafana.dev.io', disabled: false },
        { ip: '10.50.128.33', domain: 'kibana.dev.io', disabled: false },
        { ip: '10.50.128.34', domain: 'sso.dev.io', disabled: false },
        { ip: '10.50.128.35', domain: 'rabbitmq.dev.io', disabled: true },
        { ip: '10.50.128.36', domain: 'nexus.dev.io', disabled: false },
        { ip: '10.50.128.37', domain: 'minio.dev.io', disabled: false },
      ],
    },
    {
      name: '默认',
      hosts: [
        { ip: '127.0.0.1', domain: 'localhost', disabled: false },
        { ip: '127.0.0.1', domain: 'dev.local', disabled: false },
      ],
    },
    {
      name: '生产环境',
      hosts: [
        { ip: '192.168.10.11', domain: 'erp.corp.io', disabled: true },
        { ip: '192.168.10.12', domain: 'oa.corp.io', disabled: true },
      ],
    },
    {
      name: 'k8s-ingress',
      hosts: [
        { ip: '10.0.0.5', domain: 'app.example.com', disabled: false },
        { ip: '10.0.0.5', domain: 'console.example.com', disabled: false },
        { ip: '10.0.0.6', domain: 'grafana.example.com', disabled: false },
        { ip: '10.0.0.7', domain: 'api.example.com', disabled: false },
      ],
    },
  ]
  const hostsSources = [
    { name: 'k8s-ingress', url: 'https://k8s.example.com/api/hosts', lastSyncedAt: Math.floor((NOW - 22 * 60000) / 1000) },
  ]

  // ---------- wallpapers ----------
  const wallpaper = (id, title, category, tags, file, description) => ({
    id,
    title,
    url: RIG + file,
    category,
    tags,
    description,
    content: `# ${title}\n\n${description}\n\n> 每天换一张，把知识钉在桌面上。`,
  })
  const wallpapers = [
    wallpaper('w1', 'System Design 入门图谱', '系统设计', ['架构', '微服务'], 'wall-1.png', '负载均衡、缓存、消息队列与存储分层，一图串起可扩展系统的骨干。'),
    wallpaper('w2', 'SOLID 原则速记', '软件工程', ['设计模式', '原则'], 'wall-2.png', '五个原则五个颜色，写代码前扫一眼，重构时心里有底。'),
    wallpaper('w3', 'TCP 握手与挥手', '网络', ['TCP', '协议'], 'wall-3.png', '三次握手、四次挥手与时序状态，排查连接问题时最想看到的那张图。'),
    wallpaper('w4', '常用数据结构全景', '算法', ['数据结构', '面试'], 'wall-4.png', '数组、链表、树、堆、图的操作复杂度，一张全景卡片放桌面。'),
    wallpaper('w5', 'Kubernetes 核心架构', '云原生', ['K8s', '容器'], 'wall-5.png', '控制平面与工作负载的关系图，看懂 Pod、Deployment 与 Service 的协作。'),
    wallpaper('w6', 'Git 分支模型图解', '工程效率', ['Git', '工作流'], 'wall-6.png', 'feature / release / hotfix 分支流向，团队约定一目了然。'),
    wallpaper('w7', 'SQL 索引与 B+ 树', '数据库', ['MySQL', '索引'], 'wall-7.png', '为什么范围查询快？把 B+ 树钉在屏幕上天天看。'),
    wallpaper('w8', '正则表达式速查卡', '工程效率', ['正则', '工具'], 'wall-8.png', '常用模式与分组语法，写到正则不用再翻文档。'),
    wallpaper('w9', 'HTTP 状态码速查', '网络', ['HTTP', '排错'], 'wall-9.png', '2xx 到 5xx 的语义与第一反应，接口排错先扫一眼。'),
    wallpaper('w10', 'Docker 命令速查', '云原生', ['Docker', '容器'], 'wall-10.png', '镜像、容器与运维三条线的高频命令，贴在桌面上照着敲。'),
    wallpaper('w11', '设计模式三巨头', '软件工程', ['设计模式', '架构'], 'wall-11.png', '工厂、观察者、策略——掌握这三个，八成场景够用。'),
    wallpaper('w12', 'Flex / Grid 速查', '软件工程', ['CSS', '前端'], 'wall-12.png', '垂直居中、等分列、吸底栏，一屏记住常用布局写法。'),
  ]

  // ---------- podcast ----------
  const sub = (title, rss, author, cover, category, desc, payType) => ({
    title,
    rss_url: 'https://demo.jedi.dev/rss/' + rss,
    description: desc,
    image_url: RIG + cover,
    website: 'https://demo.jedi.dev/' + rss,
    categories: [category],
    language: 'zh-CN',
    copyright: title,
    author,
    owner_name: author,
    owner_email: author + '@demo.jedi.dev',
    podcast_type: 'episodic',
    pay_type: payType || 'FREE',
    paid_episode_count: 0,
  })
  const subscriptions = [
    sub('内核脉搏', 'kernel-pulse', '老王与阿帆', 'cover-1.png', '科技', '每周拆解一个底层技术话题：从内核调度到分布式一致性。'),
    sub('代码与茶', 'code-and-tea', '小柯', 'cover-2.png', '编程', '轻松聊工程实践、效率工具与程序员生活。'),
    sub('设计迷宫', 'design-maze', '阿崽', 'cover-3.png', '设计', '产品设计与交互细节漫谈，每周一个真实案例复盘。'),
    sub('增长手记', 'growth-notes', '林一舟', 'cover-4.png', '商业', '独立开发者与增长实践访谈。'),
    sub('片场休息室', 'backstage', '路客', 'cover-5.png', '闲聊', '影视幕后与创作者闲聊，适合通勤收听。'),
    sub('云端夜话', 'cloud-nights', '程一帆', 'cover-6.png', '科技', '云计算、稳定性工程与 SRE 的深夜漫谈。'),
    sub('硬核观察', 'hardcore-observe', '苏黎', 'cover-7.png', '科技', '硬核技术评论：芯片、编译器与数据库内核。'),
    sub('车间电台', 'workshop-radio', '大周', 'cover-8.png', '职场', '一线工程师的职场生存与成长手记。'),
    sub('笔耕不辍', 'writing-lab', '阿禾', 'cover-9.png', '创作', '技术写作与知识管理方法，每周更新。'),
    sub('音乐拾遗', 'music-dig', '小满', 'cover-10.png', '音乐', '挖冷门好歌，边听边聊音乐背后的故事。'),
  ]
  const episode = (podcastTitle, cover, title, daysAgo, dur, notes) => ({
    title,
    audio_url: 'https://demo.jedi.dev/audio/' + encodeURIComponent(title) + '.mp3',
    guid: 'gid-' + title,
    link: 'https://demo.jedi.dev/ep/' + encodeURIComponent(title),
    image_url: RIG + cover,
    description: notes,
    show_notes: notes,
    podcast_name: podcastTitle,
    author: 'demo',
    pub_date: iso(NOW - daysAgo * 24 * HOUR),
    duration: dur,
    chapters: [],
    is_paid: false,
  })
  const episodesByRss = {}
  episodesByRss[subscriptions[0].rss_url] = {
    pay_type: 'FREE',
    paid_episode_count: 0,
    episodes: [
      episode('内核脉搏', 'cover-1.png', 'Vol.42 把 hosts 文件还给工程师', 2, '58:12', '<p>切换环境为什么要手改 hosts？本期聊聊配置漂移、远程订阅与自动化同步。</p>'),
      episode('内核脉搏', 'cover-1.png', 'Vol.41 截图工具的后端艺术', 9, '52:40', '<p>滚动长图拼接、选区录屏与本地 OCR 的实现思路。</p>'),
      episode('内核脉搏', 'cover-1.png', 'Vol.40 AI 会话进桌面端之后', 16, '63:05', '<p>本地会话存储、技能包与 MCP 工具生态的连接方式。</p>'),
      episode('内核脉搏', 'cover-1.png', 'Vol.39 桌面应用的 Rust 栈', 23, '49:58', '<p>Tauri v2 迁移实录：权限模型、更新器与跨平台打包。</p>'),
    ],
  }
  episodesByRss[subscriptions[1].rss_url] = {
    pay_type: 'FREE',
    paid_episode_count: 0,
    episodes: [
      episode('代码与茶', 'cover-2.png', 'EP.18 我的终端里住着几个 AI', 3, '46:20', '<p>CLI agent、桌面会话与技能包如何共存。</p>'),
      episode('代码与茶', 'cover-2.png', 'EP.17 独立开发的一周', 10, '38:11', '<p>从一个想法到上架安装包的一周记录。</p>'),
    ],
  }
  episodesByRss[subscriptions[2].rss_url] = {
    pay_type: 'FREE',
    paid_episode_count: 0,
    episodes: [
      episode('设计迷宫', 'cover-3.png', 'Vol.11 暗色主题不是加个滤镜', 5, '41:07', '<p>对比度、层级与留白，暗色界面的设计方法。</p>'),
    ],
  }
  episodesByRss[subscriptions[3].rss_url] = { pay_type: 'FREE', paid_episode_count: 0, episodes: [] }
  episodesByRss[subscriptions[4].rss_url] = { pay_type: 'FREE', paid_episode_count: 0, episodes: [] }

  // ---------- AI chat sessions ----------
  const trace = [
    { id: 't1', type: 'think', status: 'done', title: '理解需求：切换测试环境解析并给出验证方式', timestamp: NOW - 5 * 60000, durationMs: 1200, stepIndex: 0 },
    { id: 't2', type: 'tool', status: 'done', title: '调用 hosts 工具写入 2 条解析', toolName: 'hosts', toolCallId: 'call-1', timestamp: NOW - 5 * 60000 + 1300, durationMs: 900, stepIndex: 1, input: { group: '测试环境', entries: ['api.dev.io → 10.50.128.32', 'grafana.dev.io → 10.50.128.32'] }, output: '已写入，分组共 5 条' },
    { id: 't3', type: 'answer', status: 'done', title: '整理输出与验证脚本', timestamp: NOW - 5 * 60000 + 2300, durationMs: 2100, stepIndex: 2 },
  ]
  const runMeta = {
    phase: 'done',
    provider: 'openai',
    model: 'gpt-5.1',
    engine: 'builtin',
    startedAt: NOW - 5 * 60000,
    completedAt: NOW - 5 * 60000 + 4300,
    totalDurationMs: 4300,
    stepLimit: 25,
    enabledTools: ['hosts', 'filesystem', 'skills'],
    toolCount: 3,
    skillsUsed: [],
    finishReason: 'stop',
    usage: { input_tokens: 1834, output_tokens: 962 },
  }
  const assistantAnswer = [
    '两件事都办好了：',
    '',
    '**1. 更新 Hosts（测试环境分组）**',
    '',
    '| 域名 | 指向 | 状态 |',
    '| --- | --- | --- |',
    '| `api.dev.io` | 10.50.128.32 | ✅ 已启用 |',
    '| `grafana.dev.io` | 10.50.128.32 | ✅ 已启用 |',
    '',
    '通过 `hosts` 工具写入「测试环境」分组，其余 3 条既有记录未受影响。',
    '',
    '**2. 验证脚本**',
    '',
    '```python',
    '# verify_hosts.py —— 校验 hosts 切换是否生效',
    'import socket',
    '',
    'DOMAINS = ["api.dev.io", "grafana.dev.io"]',
    'EXPECT = "10.50.128.32"',
    '',
    'for domain in DOMAINS:',
    '    ip = socket.gethostbyname(domain)',
    '    print(f"{\'OK\' if ip == EXPECT else \'FAIL\':<4} {domain} -> {ip}")',
    '```',
    '',
    '输出全部 `OK` 即切换生效。需要的话可存成技能，下次 `/verify-hosts` 一键复用。',
  ].join('\n')
  const sessions = [
    {
      id: 's-1',
      title: 'Hosts 切换与验证脚本',
      provider: 'openai',
      model: 'gpt-5.1',
      created_at: iso(NOW - 6 * 60000),
      updated_at: iso(NOW - 5 * 60000),
      messages: [
        { role: 'user', content: '把测试环境的 api.dev.io 和 grafana.dev.io 都指到 10.50.128.32，再给我一段验证解析是否生效的脚本', created_at: iso(NOW - 6 * 60000) },
        { role: 'assistant', content: assistantAnswer, created_at: iso(NOW - 5 * 60000), metadata: JSON.stringify({ trace, run: runMeta }) },
      ],
    },
    { id: 's-2', title: '解读 K8s ingress 503', provider: 'deepseek', model: 'deepseek-v4', created_at: iso(NOW - 1 * 24 * HOUR), updated_at: iso(NOW - 1 * 24 * HOUR), messages: [{ role: 'user', content: '昨天上线后 ingress 间歇性 503，帮我梳理排查路径', created_at: iso(NOW - 1 * 24 * HOUR) }] },
    { id: 's-3', title: '写一个滚动截图脚本', provider: 'anthropic', model: 'claude-sonnet-4.5', created_at: iso(NOW - 2 * 24 * HOUR), updated_at: iso(NOW - 2 * 24 * HOUR), messages: [] },
    { id: 's-4', title: '重构 hosts 同步逻辑', provider: 'deepseek', model: 'deepseek-r2', created_at: iso(NOW - 3 * 24 * HOUR), updated_at: iso(NOW - 3 * 24 * HOUR), messages: [] },
    { id: 's-5', title: '把周报改成表格版', provider: 'openai', model: 'gpt-5.1', created_at: iso(NOW - 4 * 24 * HOUR), updated_at: iso(NOW - 4 * 24 * HOUR), messages: [] },
    { id: 's-6', title: '解释这段 Nginx 配置', provider: 'deepseek', model: 'deepseek-v4', created_at: iso(NOW - 5 * 24 * HOUR), updated_at: iso(NOW - 5 * 24 * HOUR), messages: [] },
  ]

  // ---------- agents / models / providers / skills ----------
  const agentReports = [
    {
      id: 'claude-code',
      name: 'Claude Code',
      installations: [{ agentId: 'claude-code', host: 'windows', exe: 'claude', version: '2.1.8', status: 'ready', spawn: { kind: 'windows-cmd', exe: 'claude' } }],
    },
    {
      id: 'codex',
      name: 'Codex CLI',
      installations: [{ agentId: 'codex', host: 'wsl:Ubuntu', exe: 'codex', version: '0.55.0', status: 'ready', spawn: { kind: 'wsl', distro: 'Ubuntu', exe: 'codex' } }],
    },
    {
      id: 'gemini-cli',
      name: 'Gemini CLI',
      installations: [{ agentId: 'gemini-cli', host: 'windows', exe: 'gemini', version: '3.2.0', status: 'ready', spawn: { kind: 'windows-cmd', exe: 'gemini' } }],
    },
  ]

  const model = (id, name) => ({
    id,
    name,
    attachment: true,
    reasoning: true,
    tool_call: true,
    temperature: true,
    modalities: { input: ['text', 'image'], output: ['text'] },
    open_weights: false,
    limit: { context: 200000, output: 32000 },
  })
  const modelsProviders = {
    deepseek: { id: 'deepseek', name: 'DeepSeek', models: { 'deepseek-v4': model('deepseek-v4', 'DeepSeek V4'), 'deepseek-r2': model('deepseek-r2', 'DeepSeek R2') } },
    anthropic: { id: 'anthropic', name: 'Anthropic', models: { 'claude-sonnet-4.5': model('claude-sonnet-4.5', 'Claude Sonnet 4.5'), 'claude-haiku-4.5': model('claude-haiku-4.5', 'Claude Haiku 4.5') } },
    openai: { id: 'openai', name: 'OpenAI', models: { 'gpt-5.1': model('gpt-5.1', 'GPT-5.1'), 'gpt-5.1-mini': model('gpt-5.1-mini', 'GPT-5.1 Mini') } },
    google: { id: 'google', name: 'Google', models: { 'gemini-3-pro': model('gemini-3-pro', 'Gemini 3 Pro') } },
    moonshot: { id: 'moonshot', name: 'Moonshot AI', models: { 'kimi-k2.5': model('kimi-k2.5', 'Kimi K2.5') } },
    zhipuai: { id: 'zhipuai', name: '智谱 AI', models: { 'glm-5-air': model('glm-5-air', 'GLM-5 Air') } },
  }
  const configuredProviders = [
    { provider: 'deepseek', has_key: true },
    { provider: 'anthropic', has_key: true },
    { provider: 'openai', has_key: true },
    { provider: 'google', has_key: true },
  ]

  const skill = (name, description, icon, source) => ({ name, description, icon, source, enabled: true })
  const skills = [
    skill('commit-writer', '按 Conventional Commits 规范，从暂存区 diff 生成提交信息', 'mdi-source-commit', 'builtin'),
    skill('weekly-report', '读取日历与任务记录，汇总生成结构化周报草稿', 'mdi-calendar-text', 'user'),
    skill('sql-optimizer', '分析慢查询 SQL，给出索引建议与改写方案', 'mdi-database-search', 'builtin'),
    skill('regex-explainer', '逐段解释正则表达式的含义，并给出匹配示例', 'mdi-regex', 'user'),
    skill('k8s-triage', '根据 kubectl 输出定位 Pod 异常的常见原因', 'mdi-kubernetes', 'builtin'),
    skill('api-doc-gen', '从接口定义生成 Markdown API 文档', 'mdi-file-document-outline', 'builtin'),
    skill('release-notes', '从 Git 标签区间生成面向用户的发布说明', 'mdi-tag-outline', 'user'),
  ]
  const skillStats = {
    'commit-writer': { calls: 63, failures: 0, totalMs: 41200, lastUsedAt: NOW - 2 * HOUR, recentErrors: [] },
    'weekly-report': { calls: 12, failures: 1, totalMs: 26400, lastUsedAt: NOW - 26 * HOUR, recentErrors: [{ ts: NOW - 6 * 24 * HOUR, msg: '日历服务未授权，跳过日程汇总' }] },
    'sql-optimizer': { calls: 8, failures: 0, totalMs: 9100, lastUsedAt: NOW - 3 * 24 * HOUR, recentErrors: [] },
  }

  // ---------- screenshot gallery (basenames must match generated assets) ----------
  const screenshots = [
    { name: 'styled-api-docs.png', path: 'C:\\Users\\demo\\Pictures\\jedi\\styled-api-docs.png', size: 512304, modifiedMs: NOW - 1 * HOUR, kind: 'image' },
    { name: 'styled-terminal.png', path: 'C:\\Users\\demo\\Pictures\\jedi\\styled-terminal.png', size: 298120, modifiedMs: NOW - 5 * HOUR, kind: 'image' },
    { name: 'styled-code-card.png', path: 'C:\\Users\\demo\\Pictures\\jedi\\styled-code-card.png', size: 276540, modifiedMs: NOW - 9 * HOUR, kind: 'image' },
    { name: 'release-notes-longpage.png', path: 'C:\\Users\\demo\\Pictures\\jedi\\release-notes-longpage.png', size: 821993, modifiedMs: NOW - 26 * HOUR, kind: 'image' },
    { name: 'meeting-whiteboard.png', path: 'C:\\Users\\demo\\Pictures\\jedi\\meeting-whiteboard.png', size: 331021, modifiedMs: NOW - 29 * HOUR, kind: 'image' },
    { name: 'styled-standup.png', path: 'C:\\Users\\demo\\Pictures\\jedi\\styled-standup.png', size: 302118, modifiedMs: NOW - 31 * HOUR, kind: 'image' },
    { name: 'feature-demo.mp4', path: 'C:\\Users\\demo\\Pictures\\jedi\\feature-demo.mp4', size: 18432000, modifiedMs: NOW - 2 * 24 * HOUR, kind: 'video' },
  ]

  // ---------- command table ----------
  const commands = {
    // app / system
    'plugin:app|version': () => '0.8.1',
    get_app_info: () => ({ version: '0.8.1', name: 'Jedi' }),
    get_os_info: () => osInfo,
    ensure_jedi_dir: () => 'C:\\Users\\demo\\.jedi',
    set_icon_theme: () => null,
    'plugin:updater|check': () => null,
    'plugin:updater|download_and_install': () => null,
    'plugin:autostart|is_enabled': () => false,
    'plugin:autostart|enable': () => null,
    'plugin:autostart|disable': () => null,
    'plugin:shell|open': () => null,
    'plugin:process|relaunch': () => null,
    'plugin:http|fetch': () => Promise.reject(new Error('[rig] http fetch disabled')),
    'plugin:window|is_maximized': () => false,
    // events
    'plugin:event|listen': args => {
      callbacks.set(args?.handler, { once: false })
      return ++evSeq
    },
    'plugin:event|unlisten': () => null,
    'plugin:event|emit': () => null,
    // hosts
    read_system_hosts: () => hostsGroups,
    get_hosts_path: () => 'C:\\Windows\\System32\\drivers\\etc\\hosts',
    update_hosts_with_groups: () => 'ok',
    revert_hosts: () => 'ok',
    get_hosts_sources: () => hostsSources,
    save_hosts_source: ({ source }) => {
      const i = hostsSources.findIndex(s => s.name === source?.name)
      if (i === -1) hostsSources.push(source)
      else hostsSources[i] = source
      return hostsSources
    },
    remove_hosts_source: ({ name }) => {
      const i = hostsSources.findIndex(s => s.name === name)
      if (i !== -1) hostsSources.splice(i, 1)
      return hostsSources
    },
    fetch_remote_config: () => [hostsGroups.find(g => g.name === 'k8s-ingress')],
    // wallpaper
    get_wallpapers: () => wallpapers,
    sync_wallpapers: () => null,
    get_current_wallpaper: () => RIG + 'wall-1.png',
    set_desktop_wallpaper: () => null,
    show_in_folder: () => null,
    // podcast
    get_subscriptions: () => subscriptions,
    fetch_episodes: ({ rssUrl }) =>
      episodesByRss[rssUrl] || { pay_type: 'FREE', paid_episode_count: 0, episodes: [] },
    fetch_rss_channel: ({ rssUrl }) => subscriptions.find(s => s.rss_url === rssUrl) || subscriptions[0],
    refresh_subscription: ({ rssUrl }) => subscriptions.find(s => s.rss_url === rssUrl) || subscriptions[0],
    save_subscription: ({ sub: incoming }) => {
      const i = subscriptions.findIndex(s => s.rss_url === incoming?.rss_url)
      if (i === -1) subscriptions.unshift(incoming)
      else subscriptions[i] = incoming
      return subscriptions
    },
    remove_subscription: ({ rssUrl }) => {
      const i = subscriptions.findIndex(s => s.rss_url === rssUrl)
      if (i !== -1) subscriptions.splice(i, 1)
      return subscriptions
    },
    import_opml: () => subscriptions,
    // AI chat sessions
    list_sessions: () => sessions,
    create_session: ({ request }) => {
      const created = {
        id: 's-' + (sessions.length + 1),
        title: request?.title || '新对话',
        provider: request?.provider || 'deepseek',
        model: request?.model || 'deepseek-v4',
        created_at: iso(Date.now()),
        updated_at: iso(Date.now()),
        messages: [],
      }
      sessions.unshift(created)
      return created
    },
    update_session_title: () => null,
    delete_session: () => null,
    append_message: () => null,
    // agent hub / models
    detect_agents: () => ({ reports: agentReports, durationMs: 142 }),
    fetch_models_dev: () => modelsProviders,
    get_models_dev_provider: ({ providerId }) => modelsProviders[providerId] || null,
    get_models_for_provider: ({ providerId }) => Object.values(modelsProviders[providerId]?.models || {}),
    get_models_providers: () =>
      Object.values(modelsProviders).map(p => ({
        id: p.id,
        name: p.name,
        api: p.api,
        model_count: Object.keys(p.models).length,
        category: ['deepseek', 'anthropic', 'openai', 'google'].includes(p.id) ? 'popular' : 'other',
      })),
    search_models_dev: () => [],
    // provider keys
    list_api_key_providers: () => configuredProviders,
    has_api_key: ({ provider }) => configuredProviders.some(p => p.provider === provider),
    get_api_key: ({ provider }) => ({ provider, key: 'sk-demo-****-****' }),
    store_api_key: () => null,
    delete_api_key: () => null,
    // skills
    skills_list: () => skills,
    skills_read: ({ name }) => {
      const s = skills.find(x => x.name === name)
      return s
        ? {
            name: s.name,
            description: s.description,
            icon: s.icon,
            source: s.source,
            body: `# ${s.name}\n\n${s.description}\n\n## 使用方式\n\n在聊天输入框输入 \`/${s.name}\` 强制加载本技能，或让模型在合适的时机自动触发。`,
            files: [],
            dir: 'C:\\Users\\demo\\.jedi\\skills\\' + s.name,
          }
        : null
    },
    skills_save: ({ def }) => ({ name: def?.name || 'untitled', description: def?.description || '', icon: def?.icon || 'mdi-star-outline', source: 'user', enabled: true }),
    skills_delete: () => true,
    skill_stats_list: () => skillStats,
    skill_stats_save: () => null,
    skill_stats_clear: () => null,
    // screenshot tool
    list_screenshots: () => screenshots,
    read_image_data_url: () => RIG + 'styled-api-docs.png',
    screenshot_save_styled: () => 'C:\\Users\\demo\\Pictures\\jedi\\styled-new.png',
    get_capture_hotkey: () => 'F1',
    set_capture_hotkey: () => null,
    screenshot_toolbar_info: () => ({ mode: 'scroll', frameCount: 0, maxDurationMs: 60000 }),
    screenshot_begin: () => [],
    screenshot_delete: () => null,
    screenshot_open_file: () => null,
    screenshot_copy_text: () => null,
    // agent tools / mcp (not exercised in static shots)
    read_hosts: () => hostsGroups,
    add_hosts_entry: () => 'ok',
    remove_hosts_entry: () => 'ok',
    list_hosts_groups: () => hostsGroups.map(g => g.name),
    read_file: () => '',
    write_file: () => 'ok',
    file_exists: () => false,
    mcp_connect: () => Promise.reject(new Error('[rig] mcp disabled')),
    mcp_disconnect: () => null,
    mcp_call_tool: () => Promise.reject(new Error('[rig] mcp disabled')),
  }

  // ---------- the internals shim ----------
  window.__TAURI_INTERNALS__ = {
    metadata: {
      currentWindow: { label: 'main' },
      currentWebview: { windowLabel: 'main' },
    },
    plugins: {},
    transformCallback(callback) {
      const id = ++cbSeq
      callbacks.set(id, { callback })
      return id
    },
    convertFileSrc(filePath) {
      // Map any absolute path to the matching rig asset by basename.
      const base = String(filePath).replace(/\\/g, '/').split('/').pop() || 'image.png'
      return RIG + base
    },
    async invoke(cmd, args) {
      const fn = commands[cmd]
      if (!fn) {
        console.warn('[rig] unmocked command:', cmd, args)
        return null
      }
      return await fn(args || {})
    },
  }

  // ---------- optional per-shot actions, driven by ?rigAction= ----------
  const params = new URLSearchParams(window.location.search)
  const action = params.get('rigAction')

  function pinia() {
    const app = document.getElementById('app')
    return app && app.__vue_app__ ? app.__vue_app__.config.globalProperties.$pinia : null
  }
  function waitFor(cond, cb, timeoutMs = 15000, step = 100) {
    const t0 = Date.now()
    const timer = setInterval(() => {
      let ok = false
      try {
        ok = cond()
      } catch (e) {
        ok = false
      }
      if (ok || Date.now() - t0 > timeoutMs) {
        clearInterval(timer)
        if (ok) cb()
      }
    }, step)
  }

  if (action === 'chat-hero') {
    // Select the demo provider/model so the header chip shows real names.
    waitFor(
      () => {
        const p = pinia()
        return p && p.state.value.modelsDev && Object.keys(p.state.value.modelsDev.providersData || {}).length > 0
      },
      () => {
        const st = pinia().state.value
        st.modelsDev.selectedProviderId = 'openai'
        st.modelsDev.selectedModelId = 'gpt-5.1'
        st.modelsDev.lastSelectedProviderId = 'openai'
        st.modelsDev.lastSelectedModelId = 'gpt-5.1'
      }
    )
    // Select the demo session so the message list renders, then keep the
    // conversation scrolled to its top (short demo fits the viewport).
    waitFor(
      () => {
        const p = pinia()
        return p && ((p.state.value.aiChat && p.state.value.aiChat.sessions && p.state.value.aiChat.sessions.length) || 0) > 0
      },
      () => {
        pinia().state.value.aiChat.currentSessionId = 's-1'
        setTimeout(() => {
          const box = document.querySelector('.messages-container')
          if (box) box.scrollTop = 0
        }, 350)
      }
    )
    // Open the history panel (the header toggle carries aria-expanded).
    waitFor(
      () => !!document.querySelector('.chat-header .workers-btn[aria-expanded]'),
      () => {
        document.querySelector('.chat-header .workers-btn[aria-expanded]').click()
      }
    )
  }
  if (action === 'open-history') {
    waitFor(
      () => !!document.querySelector('.chat-header .workers-btn[aria-expanded]'),
      () => {
        document.querySelector('.chat-header .workers-btn[aria-expanded]').click()
      }
    )
  }
  window.__RIG_READY__ = true
})()
