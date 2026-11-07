#!/usr/bin/env node
/**
 * scripts/preview-installer.mjs
 * -----------------------------------------------------------------------------
 * Render a Tauri NSIS installer template (Handlebars `{{...}}` placeholders)
 * with sample data, neutralize every installation side effect, and compile it
 * with the local makensis so the resulting .exe can be driven page-by-page by
 * `scripts/capture-installer.ps1` for screenshots.
 *
 * The generated preview installer NEVER
 *   - writes the registry
 *   - copies files into $INSTDIR
 *   - creates Start Menu / desktop shortcuts
 *   - touches %LOCALAPPDATA%\Jedi, %APPDATA%\io.jedi or the installed app
 *   - launches the app from the finish page
 *   - asks the real uninstaller to run (dangerous on an up/downgrade!)
 * It only prints DetailPrint lines, animates a fake progress bar (AddSize +
 * Sleep) and writes ONE file: <out>/uninstall.exe (the preview uninstaller).
 *
 * Zero third-party dependencies (Node >= 18).
 *
 * Usage
 *   node scripts/preview-installer.mjs                     # auto: after -> before
 *   node scripts/preview-installer.mjs --template before   # rendered ref (authoritative)
 *   node scripts/preview-installer.mjs --template after --lang en
 *   node scripts/preview-installer.mjs --check             # Handlebars parity regression
 *   node scripts/preview-installer.mjs --no-compile --out .tmp/x
 *
 * Options
 *   --template <path|preset>  preset: after | before | baseline
 *                             (default: after when it exists, otherwise before)
 *                             `before`   = src-tauri/target/release/nsis/x64/installer.nsi
 *                                          (already rendered -> used verbatim)
 *                             `baseline` = .tmp/nsis-baseline/installer.nsi.template
 *                                          (pristine Tauri CLI 2.10.1 template,
 *                                           exercises the Handlebars renderer)
 *                             <path>     = any .nsi / template file
 *   --lang <zh|en|both>       default zh
 *   --out <dir>               default .tmp/installer-preview/<preset>-<lang>
 *   --arch <x64|arm64|...>    default x64
 *   --version <x.y.z>         override product version (default 0.6.0)
 *   --data <file.json>        merge extra/override values into the sample data
 *   --stub / --no-stub        default: stub ON (preview mode)
 *   --no-compile              render only
 *   --check                   render the pristine baseline template and compare it
 *                             against the committed reference .nsi; exit 1 on drift
 *   --print-data              dump the resolved sample data as JSON
 *   --quiet
 */

import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, '..');
const TMP = path.join(ROOT, '.tmp');
const BASELINE_DIR = path.join(TMP, 'nsis-baseline');
const REFERENCE_NSI = path.join(ROOT, 'src-tauri', 'target', 'release', 'nsis', 'x64', 'installer.nsi');
const AFTER_NSI = path.join(ROOT, 'src-tauri', 'windows', 'nsis', 'installer.nsi');
const SUPPORT_DIR = path.join(ROOT, 'src-tauri', 'target', 'release', 'nsis', 'x64');

/* ========================================================================== *
 * 1. tiny CLI
 * ========================================================================== */

function parseArgv(argv) {
  const o = {
    template: null, lang: 'zh', out: null, arch: 'x64', version: '0.6.0',
    data: null, stub: true, compile: true, check: false, printData: false, quiet: false,
  };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    const next = () => {
      const v = argv[++i];
      if (v === undefined) fail(`option ${a} needs a value`);
      return v;
    };
    switch (a) {
      case '--template': o.template = next(); break;
      case '--lang': o.lang = next(); break;
      case '--out': o.out = next(); break;
      case '--arch': o.arch = next(); break;
      case '--version': o.version = next(); break;
      case '--data': o.data = next(); break;
      case '--stub': o.stub = true; break;
      case '--no-stub': o.stub = false; break;
      case '--no-compile': o.compile = false; break;
      case '--compile': o.compile = true; break;
      case '--check': o.check = true; break;
      case '--print-data': o.printData = true; break;
      case '--quiet': o.quiet = true; break;
      case '-h':
      case '--help': usage(0); break;
      default: fail(`unknown option ${a} (try --help)`);
    }
  }
  if (!['zh', 'en', 'both'].includes(o.lang)) fail(`--lang must be zh|en|both (got ${o.lang})`);
  return o;
}

function usage(code) {
  const head = fs.readFileSync(fileURLToPath(import.meta.url), 'utf8').split('*/')[0];
  console.log(head.replace(/^\/\*\*?|^ \* ?/gm, '').replace(/\s+$/, ''));
  process.exit(code);
}

function fail(msg) {
  console.error(`\n[preview-installer] ERROR: ${msg}\n`);
  process.exit(1);
}

const log = (...a) => console.log('[preview-installer]', ...a);
const warn = (...a) => console.warn('[preview-installer] WARN:', ...a);

/** synchronous sleep (no deps) */
function sleepSync(ms) {
  const sab = new SharedArrayBuffer(4);
  Atomics.wait(new Int32Array(sab), 0, 0, ms);
}

/**
 * Read a template that another agent may be writing right now. Two identical
 * (size, mtime, bytes) snapshots are required, otherwise a torn read produces a
 * syntactically broken .nsi ("FunctionEnd not valid outside Function").
 */
function readStableFile(file, { tries = 10, gapMs = 500 } = {}) {
  let prev = null;
  for (let i = 0; i < tries; i++) {
    let st;
    try { st = fs.statSync(file); } catch { fail(`template not found: ${file}`); }
    const buf = fs.readFileSync(file);
    const sig = `${st.size}@${st.mtimeMs}`;
    if (prev && prev.sig === sig && prev.buf.equals(buf)) {
      if (i > 0) log(`template settled after ${i * gapMs} ms (${st.size} bytes)`);
      return { text: buf.toString('utf8'), sig };
    }
    prev = { sig, buf };
    sleepSync(gapMs);
  }
  warn(`${file} is still changing after ${tries * gapMs} ms — the render may come from a mid-write snapshot`);
  return { text: prev.buf.toString('utf8'), sig: prev.sig };
}

/* ========================================================================== *
 * 2. Handlebars subset
 *    {{key}} {{a.b}} {{this}} {{this.[1]}} {{@key}} {{@index}} {{no-escape x}}
 *    {{or a b}} {{association-description a b}} {{{raw}}}
 *    {{#if x}}..{{else}}..{{/if}}  {{#unless x}}  {{#each x}}..{{/each}}
 *    {{#each x as |item| ~}}       (block params + `~` whitespace control)
 * ========================================================================== */

const HTML_ESCAPE = { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#x27;', '`': '&#x60;', '=': '&#x3D;' };
const esc = (v) => String(v).replace(/[&<>"'`=]/g, (c) => HTML_ESCAPE[c]);
const warnings = new Set();
const warnOnce = (m) => { if (!warnings.has(m)) { warnings.add(m); warn(m); } };

function tokenizeHbs(src) {
  const tokens = [];
  let i = 0;
  let trimNextText = false;
  const pushText = (v) => { if (v) tokens.push({ type: 'text', value: v }); };
  while (i < src.length) {
    const open = src.indexOf('{{', i);
    if (open < 0) { pushText(src.slice(i)); break; }
    let text = src.slice(i, open);
    if (trimNextText) { text = text.replace(/^\s+/, ''); trimNextText = false; }
    let j = open + 2;
    let triple = false;
    if (src[j] === '{') { triple = true; j++; }
    const closer = triple ? '}}}' : '}}';
    const close = src.indexOf(closer, j);
    if (close < 0) throw new Error(`unterminated mustache at offset ${open}`);
    let body = src.slice(j, close).trim();
    let trimBefore = false;
    let trimAfter = false;
    if (body.startsWith('~')) { trimBefore = true; body = body.slice(1).trim(); }
    if (body.endsWith('~')) { trimAfter = true; body = body.slice(0, -1).trim(); }
    if (trimBefore) text = text.replace(/\s+$/, '');
    pushText(text);
    tokens.push({ type: 'tag', body, triple });
    if (trimAfter) trimNextText = true;
    i = close + closer.length;
  }
  return tokens;
}

/** split `each foo.bar as |item|` into positional tokens (quotes kept together) */
function splitArgs(str) {
  const out = [];
  let i = 0;
  while (i < str.length) {
    const c = str[i];
    if (/\s/.test(c)) { i++; continue; }
    if (c === '"' || c === "'") {
      let j = i + 1;
      let v = '';
      while (j < str.length && str[j] !== c) v += str[j++];
      out.push({ literal: true, value: v });
      i = j + 1;
      continue;
    }
    let j = i;
    let v = '';
    while (j < str.length && !/\s/.test(str[j])) v += str[j++];
    out.push({ literal: false, value: v });
    i = j;
  }
  return out;
}

function parseHbs(tokens, label = 'template') {
  const root = { type: 'root', children: [], inverse: [] };
  const stack = [{ node: root, branch: 'children' }];
  const emit = (n) => {
    const top = stack[stack.length - 1];
    top.node[top.branch].push(n);
  };
  for (const t of tokens) {
    if (t.type === 'text') { emit({ type: 'text', value: t.value }); continue; }
    const body = t.body;
    if (!body || body.startsWith('!')) continue;                       // {{! comment}}
    if (body.startsWith('#')) {
      const parts = splitArgs(body.slice(1));
      const name = parts.shift().value;
      const node = { type: 'block', name, args: parts, blockParams: [], children: [], inverse: [] };
      const asIdx = parts.findIndex((p) => !p.literal && p.value === 'as');
      if (asIdx >= 0) {
        const tail = parts.slice(asIdx + 1).map((p) => p.value).join(' ');
        const m = tail.match(/^\|([^|]*)\|$/);
        if (m) node.blockParams = m[1].trim().split(/\s+/).filter(Boolean);
        node.args = parts.slice(0, asIdx);
      }
      emit(node);
      stack.push({ node, branch: 'children' });
      continue;
    }
    if (body.startsWith('/')) {
      const name = body.slice(1).trim();
      const top = stack.pop();
      if (!top || top.node.name !== name) throw new Error(`mismatched {{/${name}}} in ${label}`);
      continue;
    }
    if (body === 'else' || body === '^') {
      const top = stack[stack.length - 1];
      if (!top || top.node.type !== 'block') throw new Error(`{{else}} outside a block in ${label}`);
      top.branch = 'inverse';
      continue;
    }
    const parts = splitArgs(body);
    emit({ type: 'mustache', helper: parts.length > 1 ? parts[0].value : null, args: parts.length > 1 ? parts.slice(1) : parts, raw: t.triple });
  }
  if (stack.length !== 1) throw new Error(`unclosed block {{#${stack[stack.length - 1].node.name}}} in ${label}`);
  return root;
}

const splitPath = (p) => p.replace(/\[([^\]]*)\]/g, '.$1').split('.').filter((s, i) => s !== '' || i === 0);

function lookup(pathStr, frame) {
  let p = pathStr.trim();
  if (p === 'this' || p === '.' || p === '') return frame.ctx;
  let f = frame;
  while (p.startsWith('../')) { p = p.slice(3); f = f.parent || f; }
  if (p === '..') return f.parent ? f.parent.ctx : undefined;
  if (p.startsWith('@')) {
    const m = p.match(/^@([\w-]+)(.*)$/);
    if (!m) return undefined;
    let v = (f.data || {})[m[1]];
    const rest = m[2].replace(/^\./, '');
    if (rest) for (const seg of splitPath(rest)) v = v == null ? undefined : v[seg];
    return v;
  }
  const segs = splitPath(p);
  let v;
  const base = segs[0];
  if (base === 'this' || base === '.') v = f.ctx;
  else if (f.local && Object.prototype.hasOwnProperty.call(f.local, base)) v = f.local[base];
  else v = f.ctx == null ? undefined : f.ctx[base];
  for (const s of segs.slice(1)) v = v == null ? undefined : v[s];
  return v;
}

const isTruthy = (v) => !(v === false || v === 0 || v === '' || v === null || v === undefined || (Array.isArray(v) && v.length === 0));
const str = (v) => (v === null || v === undefined || v === false ? '' : String(v));

/** Tauri's `association-description` helper: "<description> (<EXT>)" w/ fallbacks */
function associationDescription(description, ext) {
  const e = str(ext);
  if (isTruthy(description)) return `${str(description)} (${e.toUpperCase()})`;
  return `${e.toUpperCase()} file`;
}

function evalMustache(node, frame) {
  const args = node.args.map((a) => (a.literal ? a.value : lookup(a.value, frame)));
  const helper = node.helper;
  if (!helper) {
    const v = args[0];
    return { text: str(v), raw: node.raw };
  }
  switch (helper) {
    case 'no-escape':
    case 'raw':
      return { text: str(args[0]), raw: true };
    case 'or':
      return { text: str(args.find(isTruthy)), raw: node.raw };
    case 'and':
      return { text: str(args.every(isTruthy) ? args[args.length - 1] : ''), raw: node.raw };
    case 'association-description':
      return { text: associationDescription(args[0], args[1]), raw: node.raw };
    case 'lookup': {
      const obj = args[0];
      return { text: str(obj == null ? undefined : obj[str(args[1])]), raw: node.raw };
    }
    default:
      warnOnce(`unsupported handlebars helper {{${helper}}} -> rendered as empty (template evolved?)`);
      return { text: '', raw: node.raw };
  }
}

function renderHbs(nodes, frame, out) {
  for (const n of nodes) {
    if (n.type === 'text') { out.push(n.value); continue; }
    if (n.type === 'block') {
      if (n.name === 'if' || n.name === 'unless') {
        const v = lookup(n.args[0].value, frame);
        const ok = n.name === 'unless' ? !isTruthy(v) : isTruthy(v);
        renderHbs(ok ? n.children : n.inverse, frame, out);
        continue;
      }
      if (n.name === 'each') {
        const v = lookup(n.args[0].value, frame);
        const entries = [];
        if (Array.isArray(v)) v.forEach((item, idx) => entries.push([idx, item]));
        else if (v && typeof v === 'object') Object.keys(v).forEach((k) => entries.push([k, v[k]]));
        if (entries.length === 0) { renderHbs(n.inverse, frame, out); continue; }
        entries.forEach(([key, item], idx) => {
          const local = n.blockParams.length ? { [n.blockParams[0]]: item } : null;
          const child = {
            ctx: item,
            parent: frame,
            local,
            data: { ...frame.data, key, index: idx, first: idx === 0, last: idx === entries.length - 1 },
          };
          renderHbs(n.children, child, out);
        });
        continue;
      }
      warnOnce(`unsupported handlebars block {{#${n.name}}} -> rendered as empty`);
      continue;
    }
    const { text, raw } = evalMustache(n, frame);
    out.push(raw ? text : esc(text));
  }
}

export function renderTemplate(src, data, label = 'template') {
  const ast = parseHbs(tokenizeHbs(src), label);
  const out = [];
  renderHbs(ast.children, { ctx: data, parent: null, local: null, data: { root: data } }, out);
  return out.join('');
}

const hasHandlebars = (s) => /\{\{[#/]?[!~]?[\w@]/.test(s);

/* ========================================================================== *
 * 3. sample data (values mirror src-tauri/target/release/nsis/x64/installer.nsi)
 * ========================================================================== */

const WIN = (...p) => p.join('\\');
const LANG_FILE = {
  zh: { mui: 'SimpChinese', nsh: 'SimpChinese.nsh' },
  en: { mui: 'English', nsh: 'English.nsh' },
};

function pickExisting(candidates) {
  for (const c of candidates) if (c && fs.existsSync(c)) return c;
  return null;
}

function makensisPath() {
  const candidates = [
    process.env.MAKENSIS,
    process.env.LOCALAPPDATA && path.join(process.env.LOCALAPPDATA, 'tauri', 'NSIS', 'makensis.exe'),
    'C:\\Program Files (x86)\\NSIS\\makensis.exe',
    'C:\\Program Files\\NSIS\\makensis.exe',
  ];
  return pickExisting(candidates);
}

function pluginDir() {
  return process.env.LOCALAPPDATA
    ? path.join(process.env.LOCALAPPDATA, 'tauri', 'NSIS', 'Plugins', 'x86-unicode', 'additional')
    : '';
}

function buildData({ lang, arch, version, outDir, langNshPath }) {
  const srcTauri = path.join(ROOT, 'src-tauri');
  const modelDir = ['node_modules', '@rustors', 'model-ppocrv6-tiny'];
  // Tauri resolves resources relative to src-tauri, hence the literal `..`
  const resSrc = (rel) => WIN(srcTauri, '..', ...modelDir, ...rel.split('/'));
  const resources = {};
  resources[resSrc('LICENSE')] = [resSrc('LICENSE'), 'licenses\\rusto-model-MIT.txt'];
  resources[resSrc('models/det.rten')] = [resSrc('models/det.rten'), 'ocr\\det.rten'];
  resources[resSrc('models/dict.txt')] = [resSrc('models/dict.txt'), 'ocr\\dict.txt'];
  resources[resSrc('models/rec.rten')] = [resSrc('models/rec.rten'), 'ocr\\rec.rten'];

  const iconPath = path.join(srcTauri, 'icons', 'icon.ico');
  const sidebar = path.join(srcTauri, 'windows', 'nsis', 'jedi-change.ico');
  const header = path.join(srcTauri, 'windows', 'nsis', 'jedi-app-icon.ico');
  const mainExe = path.join(srcTauri, 'target', 'release', 'Jedi.exe');

  const missing = [];
  const existingOr = (p, what) => {
    if (fs.existsSync(p)) return p;
    missing.push(`${what}: ${p}`);
    return '';
  };

  const nshSrc = pickExisting([
    path.join(SUPPORT_DIR, LANG_FILE[lang === 'both' ? 'zh' : lang].nsh),
    path.join(BASELINE_DIR, LANG_FILE[lang === 'both' ? 'zh' : lang].nsh),
  ]);
  void nshSrc;

  const data = {
    // ---- strings Tauri substitutes into the template ----
    compression: 'lzma',
    manufacturer: 'jedi',
    product_name: 'Jedi',
    version,
    version_with_build: `${version}.0`,
    homepage: 'https://github.com/yolopunk/jedi',
    install_mode: 'currentUser',
    license: '',
    installer_icon: existingOr(iconPath, 'installer icon'),
    sidebar_image: existingOr(sidebar, 'sidebar image'),
    header_image: existingOr(header, 'header image'),
    main_binary_name: 'Jedi',
    main_binary_path: existingOr(mainExe, 'main binary'),
    bundle_id: 'io.jedi',
    copyright: '',
    out_file: 'nsis-output.exe',
    arch,
    additional_plugins_path: pluginDir(),
    allow_downgrades: 'true',
    display_language_selector: 'false',
    install_webview2_mode: 'downloadBootstrapper',
    webview2_installer_args: '/silent',
    webview2_bootstrapper_path: '',
    webview2_installer_path: '',
    minimum_webview2_version: '',
    uninstaller_sign_cmd: '',
    estimated_size: '101277',
    start_menu_folder: '',
    installer_hooks: '',
    // ---- collections ----
    languages: lang === 'both' ? ['SimpChinese', 'English'] : [LANG_FILE[lang].mui],
    language_files: (lang === 'both' ? ['zh', 'en'] : [lang]).map((l) => langNshPath(l)),
    resources_dirs: ['ocr', 'licenses'],
    resources_ancestors: ['ocr', 'licenses'],
    resources,
    binaries: {},
    file_associations: [],
    deep_link_protocols: [],
  };

  data.__meta = { lang, outDir, missing };
  return data;
}

/* ========================================================================== *
 * 4. stub / safety transform
 * ========================================================================== */

const STUB_BANNER = '; ===== PREVIEW STUB (generated by scripts/preview-installer.mjs) =====';

function stubTexts(lang) {
  const zh = {
    banner: '预览模式：不写入任何文件、注册表或快捷方式',
    webview2: '预览模式：跳过 WebView2 检测与安装（不下载、不 exec）',
    copy: '预览模式：将复制主程序 Jedi.exe',
    res: '预览模式：将复制 4 个资源文件（ocr / licenses）',
    reg: '预览模式：将写入卸载信息注册表项',
    lnk: '预览模式：将创建开始菜单与桌面快捷方式',
    done: '预览模式：安装流程演练完成，未做任何更改',
  };
  const en = {
    banner: 'PREVIEW MODE: no files, registry keys or shortcuts are written',
    webview2: 'PREVIEW MODE: WebView2 detection/installation skipped (no download, no exec)',
    copy: 'PREVIEW MODE: would copy the main binary Jedi.exe',
    res: 'PREVIEW MODE: would copy 4 resource files (ocr / licenses)',
    reg: 'PREVIEW MODE: would write the uninstall registry entry',
    lnk: 'PREVIEW MODE: would create the Start Menu and desktop shortcuts',
    done: 'PREVIEW MODE: rehearsal finished, nothing was changed',
  };
  return lang === 'zh' ? zh : en;
}

/** lines that must never execute in a preview build (commented out in place) */
const STUB_COMMENT_PATTERNS = [
  /^WriteReg(Str|DWORD|Bin|ExpandStr)\b/i,
  /^DeleteReg(Key|Value|Tree)\b/i,
  /^DelRegTree\b/i,
  /^CreateShortcut\b/i,
  /^CreateDirectory\b/i,
  /^SetOutPath\b/i,
  /^CopyFiles\b/i,
  /^Rename\b/i,
  /^Delete\b/i,
  /^RMDir\b/i,
  /^File\b/i,
  /^FileWrite\b/i,
  /^SetFileAttributes\b/i,
  /^WriteINIStr\b/i,
  /^DeleteINIStr\b/i,
  /^Exec(Wait)?\b/i,
  /^ExecShell\b/i,
  /^ShellExecAsUser\b/i,
  /^nsExec::/i,
  /^NSISdl::/i,
  /^inetc::/i,
  /^nsis_tauri_utils::(RunAsUser|KillProcess|KillProcessCurrentUser|KillProcessCurrentUserEx|FindProcess|FindProcessCurrentUser|SemverCompare)\b/i,
  /^!insertmacro\s+(CheckIfAppIsRunning|DeleteAppUserModelId|UnpinShortcut|SetLnkAppUserModelId|APP_ASSOCIATE|APP_UNASSOCIATE)\b/i,
  /^!insertmacro\s+MUI_STARTMENU_WRITE_(BEGIN|END)\b/i,
];

/** functions whose body is replaced wholesale (can't be made safe line by line) */
const FUNCTION_NOOPS = {
  PageReinstall: ['; the maintenance page is skipped: on an up/downgrade the real code',
    '; would ExecWait the *installed* uninstaller and really remove the app',
    'Abort'],
  PageLeaveReinstall: [],
  RunMainBinary: [],
  CreateOrUpdateDesktopShortcut: [],
  CreateOrUpdateStartMenuShortcut: [],
  '.onInstSuccess': [],
  JediEnsureWebView2: ['StrCpy $JediWebView2Failed 0', 'StrCpy $JediWebView2Error ""'],
};

function replaceSection(text, name, body) {
  const re = new RegExp(String.raw`(^Section\s+${name}\b[^\r\n]*\r?\n)([\s\S]*?)(^SectionEnd\s*$)`, 'm');
  if (!re.test(text)) return { text, hit: false };
  return { text: text.replace(re, (m, head, _old, end) => `${head}${body}${end}`), hit: true };
}

function replaceFunction(text, name, body) {
  const escName = name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const re = new RegExp(String.raw`(^Function\s+${escName}\s*\r?\n)([\s\S]*?)(^FunctionEnd\s*$)`, 'm');
  if (!re.test(text)) return { text, hit: false };
  return { text: text.replace(re, (m, head, _old, end) => `${head}${body}${end}`), hit: true };
}

const isActiveLine = (line) => {
  const t = stripNsisComment(line).trim();
  if (!t) return false;
  return !/^[#;]/.test(t);
};

function applyStub(text, { lang, outDir }) {
  const T = stubTexts(lang);
  const steps = [];
  const uninstPath = WIN(outDir, 'uninstall.exe');
  const sleepMs = 380;
  const lines = text.split(/\r?\n/);
  const out = [];
  let commented = 0;
  let sleeps = 0;
  let quits = 0;

  for (let i = 0; i < lines.length; i++) {
    const raw = lines[i];
    const code = stripNsisComment(raw).trim();
    const indent = (raw.match(/^\s*/) || [''])[0];

    // 1. the one file a preview build is allowed to write
    if (/^WriteUninstaller\b/i.test(code)) {
      out.push(`${indent}WriteUninstaller "${uninstPath}"`);
      continue;
    }
    // 1b. UI resources extracted into $PLUGINSDIR (installer's own logo / icon)
    //     are safe to embed: they never touch the system — keep them executable
    if (/^File\s+"\/oname=\$PLUGINSDIR\\/i.test(code)) {
      out.push(raw);
      continue;
    }
    // 2. neutralize every mutating command in place, keeping the UI/report code
    if (code && STUB_COMMENT_PATTERNS.some((re) => re.test(code))) {
      out.push(`${indent}; [preview-stub] ${code}`);
      commented++;
      continue;
    }
    out.push(raw);
    // 3. give the progress UI time to paint between reported steps
    if (/^Call\s+(un\.)?(Jedi)?ReportStep\b/.test(code)) {
      out.push(`${indent}Sleep ${sleepMs}`);
      sleeps++;
    }
    // 4. never let the window vanish before the screenshot tooling can catch the
    //    final state: delay every Quit (the primary action Quits right after the
    //    "done" state is painted, which is too fast for a screenshot)
    if (/^Quit\b/i.test(code)) {
      out.splice(out.length - 1, 0, `${indent}; [preview-stub] keep the window alive for capture`);
      out.splice(out.length - 1, 0, `${indent}Sleep 1500`);
      quits++;
    }
  }
  text = out.join('\n');
  steps.push({ kind: 'pass', name: `commented ${commented} mutating command(s), injected ${sleeps} UI pause(s), delayed ${quits} Quit`, hit: true });

  // 4. whole-body replacements for the dangerous / non-salvageable functions
  for (const [name, bodyLines] of Object.entries(FUNCTION_NOOPS)) {
    const body = [`  ${STUB_BANNER}`, ...bodyLines.map((l) => `  ${l}`), ''].join('\n');
    const r = replaceFunction(text, name, body);
    text = r.text;
    steps.push({ kind: 'function', name, hit: r.hit });
  }

  // 5. sections: EarlyChecks/WebView2 are pure logic, Install/Uninstall may become empty
  const sectionStub = [`  ${STUB_BANNER}`, `  ; ${T.webview2}`, ''].join('\n');
  for (const name of ['EarlyChecks', 'WebView2']) {
    const r = replaceSection(text, name, `  ${STUB_BANNER}\n`);
    if (r.hit) steps.push({ kind: 'section', name, hit: true });
    text = r.text;
  }

  // classic (pre-redesign) templates do the work inline in Section Install/Uninstall:
  // if their body went empty after neutralisation, fake a lively progress page again
  for (const [name, fake] of [['Install', [
    `  ${STUB_BANNER}`,
    `  DetailPrint "${T.banner}"`,
    '  Sleep 150',
    `  DetailPrint "${T.webview2}"`,
    '  AddSize 200',
    '  Sleep 350',
    `  DetailPrint "${T.copy}"`,
    '  AddSize 45000',
    '  Sleep 650',
    `  DetailPrint "${T.res}"`,
    '  AddSize 5600',
    '  Sleep 550',
    `  DetailPrint "${T.reg}"`,
    '  AddSize 100',
    '  Sleep 400',
    `  DetailPrint "${T.lnk}"`,
    '  AddSize 80',
    '  Sleep 400',
    `  DetailPrint "${T.done}"`,
    '  AddSize 40',
    '  Sleep 250',
    '  ; the only file this preview build ever writes:',
    `  WriteUninstaller "${uninstPath}"`,
    '',
  ]], ['Uninstall', [`  ${STUB_BANNER}`, `  ; ${T.banner}`, '']]]) {
    const re = new RegExp(String.raw`^Section\s+${name}\b[^\r\n]*\r?\n([\s\S]*?)^SectionEnd\s*$`, 'm');
    const m = text.match(re);
    if (!m) { steps.push({ kind: 'section', name, hit: false }); continue; }
    const bodyLines = m[1].split(/\r?\n/);
    const hasActive = bodyLines.some(isActiveLine);
    if (hasActive) {
      // the template drives the UI itself (single-window design) - keep its calls,
      // they are already safe: the real work was neutralised above
      steps.push({ kind: 'section', name, hit: true });
      continue;
    }
    text = text.replace(re, (mm, _body) => `${mm.split('\n')[0]}\n${fake.join('\n')}SectionEnd`);
    steps.push({ kind: 'section', name, hit: true, note: 'injected fake progress' });
  }
  void sectionStub;
  return { text, steps };
}

/* ---- safety audit: refuse to compile a preview that could mutate the box ---- */
const FORBIDDEN_COMMANDS = [
  'WriteRegStr', 'WriteRegDWORD', 'WriteRegBin', 'WriteRegExpandStr', 'DelRegTree',
  'DeleteRegKey', 'DeleteRegValue', 'DeleteRegTree',
  'CreateShortcut', 'CreateDirectory', 'SetOutPath', 'CopyFiles', 'Rename',
  'Delete', 'RMDir', 'RmDir', 'File', 'FileWrite', 'SetFileAttributes',
  'ExecWait', 'Exec', 'ExecShell', 'ShellExecAsUser', 'nsExec',
  'WriteINIStr', 'DeleteINIStr', 'WriteUninstaller',
];
const FORBIDDEN_MACROS = ['CheckIfAppIsRunning', 'DeleteAppUserModelId', 'UnpinShortcut', 'SetLnkAppUserModelId'];

function stripNsisComment(line) {
  // remove `;` comments that are not inside a double quoted string
  let inStr = false;
  for (let i = 0; i < line.length; i++) {
    const c = line[i];
    if (c === '"') inStr = !inStr;
    else if (c === ';' && !inStr) return line.slice(0, i);
  }
  return line;
}

function auditPreview(text, { allowedWriteUninstaller }) {
  const findings = [];
  const lines = text.split(/\r?\n/);
  lines.forEach((raw, idx) => {
    const line = stripNsisComment(raw).trim();
    if (!line) return;
    const cmd = line.match(/^([A-Za-z_][\w]*)\b/);
    if (cmd && FORBIDDEN_COMMANDS.includes(cmd[1])) {
      // UI 资源：写入 $PLUGINSDIR 的安装器自带图标（logo / 更改铅笔）不落系统盘，放行
      if (cmd[1] === 'File' && /\$PLUGINSDIR/i.test(line)) return;
      if (cmd[1] === 'WriteUninstaller') {
        if (allowedWriteUninstaller && line.includes(allowedWriteUninstaller)) return;
      }
      findings.push({ line: idx + 1, text: line });
      return;
    }
    const mac = line.match(/^!insertmacro\s+(\w+)/);
    if (mac && FORBIDDEN_MACROS.includes(mac[1])) findings.push({ line: idx + 1, text: line });
  });
  return findings;
}

/* ========================================================================== *
 * 5. language normalisation (both for rendered templates and for .nsi files
 *    that are already rendered verbatim, e.g. the before baseline)
 * ========================================================================== */

/**
 * Make sure the script declares exactly the languages we want and includes our
 * own generated language file. Works both for freshly rendered templates and for
 * .nsi files that were already rendered by Tauri (e.g. the `before` baseline,
 * which hard-codes `!insertmacro MUI_LANGUAGE "English"` and the absolute path
 * of the build-time English.nsh).
 */
function forceLanguage(text, lang, langNshPath) {
  const wanted = lang === 'both' ? ['SimpChinese', 'English'] : [LANG_FILE[lang].mui];
  const keepSymbol = lang === 'zh' ? '${LANG_SIMPCHINESE}' : lang === 'en' ? '${LANG_ENGLISH}' : null;
  const nshPaths = (lang === 'both' ? ['zh', 'en'] : [lang]).map((l) => langNshPath(l));
  const out = [];
  const seen = new Set();
  let inLangSection = false;
  let includesEmitted = false;
  const dropped = [];

  const emitIncludes = () => {
    if (includesEmitted) return;
    includesEmitted = true;
    for (const p of nshPaths) out.push(`  !include "${p}"`);
  };

  for (const line of text.split(/\r?\n/)) {
    // drop LangString definitions of the languages that are not compiled in
    const mStr = line.match(/^\s*LangString\s+\S+\s+\$\{(LANG_[A-Z0-9_]+)\}/i);
    if (mStr && keepSymbol && `\${${mStr[1]}}` !== keepSymbol) { dropped.push(line.trim()); continue; }

    const mLang = line.match(/^(\s*)!insertmacro\s+MUI_LANGUAGE\s+"?([\w]+)"?\s*$/);
    if (mLang) {
      const name = mLang[2];
      if (!wanted.includes(name) || seen.has(name)) continue;
      seen.add(name);
      out.push(line);
      continue;
    }
    if (/^\s*!insertmacro\s+MUI_RESERVEFILE_LANGDLL\s*$/.test(line)) {
      for (const w of wanted) {
        if (!seen.has(w)) { out.push(`!insertmacro MUI_LANGUAGE "${w}"`); seen.add(w); }
      }
      out.push(line);
      inLangSection = true;
      continue;
    }
    if (inLangSection && /^\s*!include\s+"[^"]*\.nsh"\s*$/.test(line)) {
      emitIncludes();
      continue;
    }
    out.push(line);
  }
  if (!includesEmitted) emitIncludes();
  if (dropped.length) log(`language filter: dropped ${dropped.length} LangString line(s) for non-compiled languages`);
  return out.join('\n');
}

/* ========================================================================== *
 * 6. compile
 * ========================================================================== */

function compile(nsiPath, outDir, { quiet }) {
  const mkn = makensisPath();
  if (!mkn) fail('makensis.exe not found (set $env:MAKENSIS or install NSIS)');
  log(`makensis: ${mkn}`);
  const res = spawnSync(mkn, ['/V2', '/INPUTCHARSET', 'UTF8', path.basename(nsiPath)], {
    cwd: outDir, encoding: 'buffer', maxBuffer: 64 * 1024 * 1024,
  });
  const stdout = res.stdout ? res.stdout.toString('utf8') : '';
  const stderr = res.stderr ? res.stderr.toString('utf8') : '';
  if (!quiet) {
    if (stdout.trim()) console.log(stdout.trimEnd());
    if (stderr.trim()) console.error(stderr.trimEnd());
  }
  if (res.status !== 0) {
    console.error('\n================ makensis FAILED ================');
    console.error(`exit code: ${res.status}${res.signal ? ` signal: ${res.signal}` : ''}`);
    console.error('--- stdout ---\n' + stdout);
    console.error('--- stderr ---\n' + stderr);
    fail('makensis compilation failed');
  }
  const exe = path.join(outDir, 'nsis-output.exe');
  if (!fs.existsSync(exe)) fail(`makensis reported success but ${exe} is missing`);
  return { exe, output: stdout };
}

/* ========================================================================== *
 * 7. --check : Handlebars parity against the committed reference render
 * ========================================================================== */

function normalizeNsi(text) {
  return text
    .replace(/^\ufeff/, '')
    .replace(/\r\n/g, '\n')
    .replace(/\\{2,}/g, '\\')
    .split('\n')
    .map((l) => l.replace(/\s+$/, ''))
    .join('\n');
}

function checkParity(opts) {
  const tplPath = path.join(BASELINE_DIR, 'installer.nsi.template');
  if (!fs.existsSync(tplPath)) fail(`baseline template missing: ${tplPath}`);
  if (!fs.existsSync(REFERENCE_NSI)) fail(`reference render missing: ${REFERENCE_NSI}`);

  const tpl = fs.readFileSync(tplPath, 'utf8');
  const data = buildData({
    lang: 'en', arch: 'x64', version: '0.4.0', outDir: BASELINE_DIR,
    langNshPath: (l) => path.join(SUPPORT_DIR, LANG_FILE[l].nsh),
  });
  // reproduce exactly what the 0.4.0 build had (see src-tauri/target/release/nsis/x64/installer.nsi)
  data.homepage = '';
  data.copyright = '';
  data.version = '0.4.0';
  data.version_with_build = '0.4.0.0';
  data.sidebar_image = '';
  data.header_image = '';
  const got = renderTemplate(tpl, data, 'baseline');

  const ref = fs.readFileSync(REFERENCE_NSI, 'utf8');
  const a = normalizeNsi(ref).split('\n').filter((l) => l.trim() !== '');
  const b = normalizeNsi(got).split('\n').filter((l) => l.trim() !== '');

  const key = (arr) => { const m = new Map(); for (const l of arr) m.set(l, (m.get(l) || 0) + 1); return m; };
  const ka = key(a), kb = key(b);
  const missing = [], extra = [];
  for (const [l, n] of ka) { const d = n - (kb.get(l) || 0); for (let i = 0; i < d; i++) missing.push(l); }
  for (const [l, n] of kb) { const d = n - (ka.get(l) || 0); for (let i = 0; i < d; i++) extra.push(l); }

  const orderedDiffs = [];
  for (let i = 0; i < Math.max(a.length, b.length); i++) {
    if (a[i] !== b[i]) { orderedDiffs.push({ line: i + 1, ref: a[i], got: b[i] }); }
  }

  console.log('\n==== Handlebars parity check (baseline template vs committed render) ====');
  console.log(`template : ${tplPath}`);
  console.log(`reference: ${REFERENCE_NSI}`);
  console.log(`lines    : ref=${a.length} rendered=${b.length}`);
  console.log(`multiset : missing=${missing.length} extra=${extra.length}`);
  if (orderedDiffs.length) {
    console.log(`positional order differences: ${orderedDiffs.length} (HashMap iteration order in the real build is not stable)`);
    for (const d of orderedDiffs.slice(0, 12)) {
      console.log(`  ~ line ${d.line}\n      ref: ${d.ref || '<none>'}\n      got: ${d.got || '<none>'}`);
    }
    if (orderedDiffs.length > 12) console.log(`  ... ${orderedDiffs.length - 12} more`);
  }
  if (missing.length || extra.length) {
    for (const l of missing.slice(0, 20)) console.log(`  - only in reference: ${l}`);
    for (const l of extra.slice(0, 20)) console.log(`  + only in rendered : ${l}`);
    console.log('\nRESULT: MISMATCH');
    process.exit(1);
  }
  console.log(orderedDiffs.length ? '\nRESULT: PASS (content identical, ordering differs only)' : '\nRESULT: PASS (byte-identical after normalisation)');
  process.exit(0);
}

/* ========================================================================== *
 * 8. main
 * ========================================================================== */

function resolveTemplate(arg) {
  if (!arg) {
    if (fs.existsSync(AFTER_NSI)) return { kind: 'preset', preset: 'after', file: AFTER_NSI };
    return { kind: 'preset', preset: 'before', file: REFERENCE_NSI };
  }
  if (arg === 'after') return { kind: 'preset', preset: 'after', file: AFTER_NSI };
  if (arg === 'before') return { kind: 'preset', preset: 'before', file: REFERENCE_NSI };
  if (arg === 'baseline') return { kind: 'preset', preset: 'baseline', file: path.join(BASELINE_DIR, 'installer.nsi.template') };
  const abs = path.isAbsolute(arg) ? arg : path.resolve(ROOT, arg);
  let preset = 'custom';
  if (abs === AFTER_NSI) preset = 'after';
  else if (abs === REFERENCE_NSI) preset = 'before';
  else if (abs === path.join(BASELINE_DIR, 'installer.nsi.template')) preset = 'baseline';
  return { kind: 'path', preset, file: abs };
}

function ensureSupportFile(name, outDir) {
  const dest = path.join(outDir, name);
  const src = pickExisting([
    path.join(SUPPORT_DIR, name),
    path.join(TMP, 'installer-before', name),
    path.join(BASELINE_DIR, name),
  ]);
  if (!src) {
    fail(`support file ${name} not found. It is produced by a real \`pnpm tauri build --bundles nsis\`.\n` +
      `Searched:\n  ${SUPPORT_DIR}\n  ${path.join(TMP, 'installer-before')}\n  ${BASELINE_DIR}`);
  }
  fs.copyFileSync(src, dest);
  return dest;
}

function writeLangNsh(outDir, lang) {
  const langs = lang === 'both' ? ['zh', 'en'] : [lang];
  for (const l of langs) {
    const name = LANG_FILE[l].nsh;
    const src = pickExisting([path.join(BASELINE_DIR, name), path.join(SUPPORT_DIR, name)]);
    if (!src) fail(`language file ${name} missing in ${BASELINE_DIR}`);
    fs.copyFileSync(src, path.join(outDir, name));
  }
}

function main() {
  const opts = parseArgv(process.argv.slice(2));
  if (opts.check) checkParity(opts);

  const tpl = resolveTemplate(opts.template);
  if (!fs.existsSync(tpl.file)) {
    if (tpl.preset === 'after') {
      fail(`src-tauri/windows/nsis/installer.nsi does not exist yet (is the installer-template task done?)`);
    }
    fail(`template not found: ${tpl.file}`);
  }

  const outDir = path.resolve(ROOT, opts.out || path.join(TMP, 'installer-preview', `${tpl.preset}-${opts.lang}`));
  fs.rmSync(outDir, { recursive: true, force: true });
  fs.mkdirSync(outDir, { recursive: true });

  const langNshPath = (l) => path.join(outDir, LANG_FILE[l].nsh);
  const data = buildData({ lang: opts.lang, arch: opts.arch, version: opts.version, outDir, langNshPath });
  if (opts.data) {
    const extra = JSON.parse(fs.readFileSync(path.resolve(ROOT, opts.data), 'utf8'));
    Object.assign(data, extra);
  }
  if (opts.printData) {
    const { __meta, ...rest } = data;
    console.log(JSON.stringify(rest, null, 2));
    if (!opts.compile) process.exit(0);
  }

  const raw = readStableFile(tpl.file).text;
  const templated = hasHandlebars(raw);
  let text = templated ? renderTemplate(raw, data, path.basename(tpl.file)) : raw;
  if (templated && raw === text) warn('template has {{...}} but rendered output is unchanged — check the renderer');

  text = forceLanguage(text, opts.lang, langNshPath);

  // ---- language self-check -------------------------------------------------
  // A single-language build must not keep LangString lines for the other
  // language: makensis would fall back to "using language id <first>" for them
  // (warning 7025) and the last definition would silently overwrite ours (6030),
  // which is exactly how English text ends up in a Chinese installer.
  if (opts.lang !== 'both') {
    const other = opts.lang === 'zh' ? 'LANG_ENGLISH' : 'LANG_SIMPCHINESE';
    const leakRe = new RegExp(String.raw`^\s*LangString\s+\S+\s+\$\{${other}\}`, 'i');
    const leaks = text.split(/\r?\n/).filter((l) => leakRe.test(l));
    if (leaks.length) {
      console.error('Leftover foreign-language LangString lines:');
      for (const l of leaks.slice(0, 10)) console.error('  ' + l.trim());
      fail(`${leaks.length} LangString line(s) for \${${other}} survived the --lang ${opts.lang} filter`);
    }
    const decls = text.split(/\r?\n/).filter((l) => /^\s*!insertmacro\s+MUI_LANGUAGE\b/.test(l));
    if (decls.length !== 1) fail(`expected exactly one MUI_LANGUAGE in a --lang ${opts.lang} build, found ${decls.length}`);
    log(`language self-check: PASS (1 language, 0 leftover \${${other}} LangString lines)`);
  }

  writeLangNsh(outDir, opts.lang);
  ensureSupportFile('utils.nsh', outDir);
  ensureSupportFile('FileAssociation.nsh', outDir);

  const uninstPath = path.join(outDir, 'uninstall.exe');
  let stubSteps = [];
  if (opts.stub) {
    const r = applyStub(text, { lang: opts.lang === 'en' ? 'en' : 'zh', outDir });
    text = r.text;
    stubSteps = r.steps;
    const missed = stubSteps.filter((s) => !s.hit);
    if (missed.length) warn(`stub targets not found in template: ${missed.map((s) => s.name).join(', ')}`);

    const findings = auditPreview(text, { allowedWriteUninstaller: uninstPath });
    if (findings.length) {
      console.error('\n===== SAFETY AUDIT FAILED =====');
      console.error('The preview installer would still perform these side effects:');
      for (const f of findings) console.error(`  line ${f.line}: ${f.text}`);
      console.error('\nRefusing to compile. Fix the stub rules in scripts/preview-installer.mjs.');
      process.exit(2);
    }
    log('safety audit: PASS (no file/registry/shortcut mutation left in the script)');
  }

  const nsiOut = path.join(outDir, 'installer.nsi');
  fs.writeFileSync(nsiOut, text.replace(/\n/g, '\r\n'), 'utf8');

  log(`template : ${tpl.file}${templated ? ' (handlebars)' : ' (already rendered)'}`);
  log(`lang     : ${opts.lang}  arch: ${opts.arch}  version: ${data.version}`);
  log(`out dir  : ${outDir}`);
  log(`stub     : ${opts.stub ? 'ON' : 'OFF (preview build WILL write to the machine!)'}`);
  if (opts.stub) log(`stub steps: ${stubSteps.filter((s) => s.hit).length}/${stubSteps.length} applied`);
  if (data.__meta.missing.length) {
    warn('missing assets (their !define will be empty -> UI falls back to defaults):');
    for (const m of data.__meta.missing) warn(`  - ${m}`);
  }
  log(`wrote    : ${nsiOut}`);

  if (!opts.compile) {
    log('--no-compile: skipping makensis');
    return;
  }
  const { exe } = compile(nsiOut, outDir, opts);
  const sizeMb = (fs.statSync(exe).size / 1048576).toFixed(1);
  log(`built    : ${exe} (${sizeMb} MB)`);
  if (opts.stub) {
    const okUn = fs.existsSync(uninstPath);
    log(`uninstaller (preview): ${uninstPath}${okUn ? '' : '  <-- not written yet, run the installer first'}`);
  }
  console.log(`\nNEXT: pwsh -File scripts/capture-installer.ps1 -Exe "${exe}" -OutDir .tmp/installer-shots -Prefix ${tpl.preset} -Lang ${opts.lang}`);
}

main();
