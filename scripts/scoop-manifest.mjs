#!/usr/bin/env node
/**
 * 刷新 Scoop 清单（bucket/jedi.json）里的 version / url / hash。
 *
 * 用法：
 *   node scripts/scoop-manifest.mjs                      # 按 tauri.conf.json 的版本刷新
 *   node scripts/scoop-manifest.mjs --version 0.9.2      # 指定版本
 *   node scripts/scoop-manifest.mjs --asset ./Jedi_0.9.2_x64-setup.exe
 *                                                        # 用本地安装包算 hash（离线/自建包）
 *   node scripts/scoop-manifest.mjs --check              # 只校验，清单过期时 exit 1
 *
 * hash 的来源顺序：
 *   1. --asset 指定的本地文件
 *   2. GitHub Releases API 的 asset digest（sha256:...）
 *   3. 下载安装包后本地计算（CI 里网络好，一般不会走到这一步）
 *
 * 发布流程：`pnpm release` 打 tag → release.yml 构建并发布 → scoop.yml 在
 * release published 之后调用本脚本并提交清单。
 */
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { Readable } from 'node:stream';
import { pipeline } from 'node:stream/promises';
import { fileURLToPath } from 'node:url';

const projectRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const manifestPath = path.join(projectRoot, 'bucket', 'jedi.json');
const tauriConfigPath = path.join(projectRoot, 'src-tauri', 'tauri.conf.json');

function parseArgs(argv) {
  const options = { version: null, asset: null, check: false, dryRun: false, help: false };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--version' || arg === '-v') {
      options.version = argv[++i];
    } else if (arg === '--asset' || arg === '-a') {
      options.asset = argv[++i];
    } else if (arg === '--check') {
      options.check = true;
    } else if (arg === '--dry-run') {
      options.dryRun = true;
    } else if (arg === '--help' || arg === '-h') {
      options.help = true;
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }
  return options;
}

function usage() {
  console.log(`
Usage: node scripts/scoop-manifest.mjs [options]

Options:
  -v, --version <x.y.z>   Version to publish (default: src-tauri/tauri.conf.json)
  -a, --asset <file>      Calculate the hash from a local installer
      --check             Exit with code 1 when the manifest is out of date
      --dry-run           Print the updated manifest instead of writing it
  -h, --help              Show this help
`);
}

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

function readProjectVersion() {
  return readJson(tauriConfigPath).version;
}

function resolveRepository(manifest) {
  // 在 Actions 里以当前仓库为准（fork 也能生成自己的清单），本地运行则看 homepage。
  if (process.env.GITHUB_REPOSITORY) {
    return process.env.GITHUB_REPOSITORY;
  }
  const match = /github\.com\/([^/]+)\/([^/#?]+)/.exec(manifest.homepage ?? '');
  if (!match) {
    throw new Error(`Cannot derive the GitHub repository from homepage: ${manifest.homepage}`);
  }
  return `${match[1]}/${match[2].replace(/\.git$/, '')}`;
}

async function hashFile(file) {
  const hash = crypto.createHash('sha256');
  await pipeline(fs.createReadStream(file), hash);
  return hash.digest('hex');
}

async function hashRemote(url) {
  const response = await fetch(url, { redirect: 'follow' });
  if (!response.ok) {
    throw new Error(`Download failed (${response.status} ${response.statusText}): ${url}`);
  }
  const hash = crypto.createHash('sha256');
  await pipeline(Readable.fromWeb(response.body), hash);
  return hash.digest('hex');
}

async function digestFromApi(repository, version, assetName) {
  const headers = { Accept: 'application/vnd.github+json', 'User-Agent': 'jedi-scoop-manifest' };
  if (process.env.GITHUB_TOKEN) {
    headers.Authorization = `Bearer ${process.env.GITHUB_TOKEN}`;
  }
  const response = await fetch(
    `https://api.github.com/repos/${repository}/releases/tags/v${version}`,
    { headers }
  );
  if (response.status === 404) {
    return null;
  }
  if (!response.ok) {
    throw new Error(`GitHub API error (${response.status} ${response.statusText})`);
  }
  const release = await response.json();
  const asset = (release.assets ?? []).find(item => item.name === assetName);
  if (!asset) {
    throw new Error(`Release v${version} has no asset named ${assetName}`);
  }
  const digest = typeof asset.digest === 'string' ? asset.digest : '';
  return /^sha256:[0-9a-f]{64}$/i.test(digest) ? digest.slice('sha256:'.length).toLowerCase() : null;
}

function serialise(manifest) {
  return `${JSON.stringify(manifest, null, 2)}\n`;
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  if (options.help) {
    usage();
    return 0;
  }

  const manifest = readJson(manifestPath);
  const version = (options.version ?? readProjectVersion()).replace(/^v/, '');
  if (!/^[0-9]+\.[0-9]+\.[0-9]+$/.test(version)) {
    throw new Error(`Invalid version: ${version}`);
  }

  const repository = resolveRepository(manifest);
  const assetName = `Jedi_${version}_x64-setup.exe`;
  const url = `https://github.com/${repository}/releases/download/v${version}/${assetName}`;

  let hash;
  if (options.asset) {
    const assetPath = path.resolve(projectRoot, options.asset);
    if (!fs.existsSync(assetPath)) {
      throw new Error(`Asset not found: ${assetPath}`);
    }
    console.log(`Hashing local asset: ${assetPath}`);
    hash = await hashFile(assetPath);
  } else {
    hash = await digestFromApi(repository, version, assetName).catch(error => {
      // 匿名 API 容易撞限流；拿不到 digest 就退化成下载安装包自己算。
      console.warn(`Could not read the release digest: ${error.message}`);
      return null;
    });
    if (!hash) {
      console.warn('Downloading the installer to compute the hash...');
      hash = await hashRemote(url);
    }
  }

  const updated = {
    ...manifest,
    version,
    architecture: {
      ...manifest.architecture,
      '64bit': {
        ...manifest.architecture?.['64bit'],
        url,
        hash: `sha256:${hash}`
      }
    }
  };

  const next = serialise(updated);
  const current = fs.existsSync(manifestPath) ? fs.readFileSync(manifestPath, 'utf8') : '';
  const changed = current !== next;

  console.log(`Version: ${version}`);
  console.log(`URL:     ${url}`);
  console.log(`SHA256:  ${hash}`);

  if (options.check) {
    if (changed) {
      console.error('bucket/jedi.json is out of date. Run `pnpm scoop:manifest` to refresh it.');
      return 1;
    }
    console.log('bucket/jedi.json is up to date.');
    return 0;
  }

  if (!changed) {
    console.log('bucket/jedi.json already up to date, nothing to write.');
    return 0;
  }

  if (options.dryRun) {
    console.log('\n--- bucket/jedi.json (dry run) ---');
    console.log(next);
    return 0;
  }

  fs.writeFileSync(manifestPath, next);
  console.log(`Updated ${path.relative(projectRoot, manifestPath)}`);
  return 0;
}

main()
  .then(code => process.exit(code))
  .catch(error => {
    console.error(`\nScoop manifest update failed: ${error.message}`);
    process.exit(1);
  });
