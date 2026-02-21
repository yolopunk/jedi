import fs from 'fs';
import path from 'path';
import { execSync } from 'child_process';
import readline from 'readline';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const filesToUpdate = [
  {
    path: 'package.json',
    regex: /"version":\s*"([0-9.]+)"/,
    replacement: '"version": "NEW_VERSION"'
  },
  {
    path: 'src-tauri/tauri.conf.json',
    regex: /"version":\s*"([0-9.]+)"/,
    replacement: '"version": "NEW_VERSION"'
  },
  {
    path: 'src-tauri/Cargo.toml',
    regex: /^version\s*=\s*"([0-9.]+)"/m,
    replacement: 'version = "NEW_VERSION"'
  }
];

function getCurrentVersion() {
  const content = fs.readFileSync('package.json', 'utf8');
  const match = content.match(/"version":\s*"([0-9.]+)"/);
  return match ? match[1] : null;
}

function updateFile(filePath, regex, replacement, newVersion) {
  const absolutePath = path.resolve(process.cwd(), filePath);
  if (!fs.existsSync(absolutePath)) {
    console.error(`File not found: ${filePath}`);
    return false;
  }
  
  let content = fs.readFileSync(absolutePath, 'utf8');
  const newContent = content.replace(regex, replacement.replace('NEW_VERSION', newVersion));
  
  if (content === newContent) {
    console.warn(`No changes made to ${filePath} (regex might not match or version is same)`);
  } else {
    fs.writeFileSync(absolutePath, newContent);
    console.log(`Updated ${filePath}`);
  }
  return true;
}

function incrementVersion(version, type) {
  const parts = version.split('.').map(Number);
  if (type === 'major') {
    parts[0]++;
    parts[1] = 0;
    parts[2] = 0;
  } else if (type === 'minor') {
    parts[1]++;
    parts[2] = 0;
  } else if (type === 'patch') {
    parts[2]++;
  }
  return parts.join('.');
}

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout
});

const currentVersion = getCurrentVersion();
console.log(`Current version: ${currentVersion}`);

const args = process.argv.slice(2);
let newVersion = null;

if (args.length > 0) {
  if (['major', 'minor', 'patch'].includes(args[0])) {
    newVersion = incrementVersion(currentVersion, args[0]);
  } else if (/^[0-9]+\.[0-9]+\.[0-9]+$/.test(args[0])) {
    newVersion = args[0];
  }
}

if (newVersion) {
  runRelease(newVersion);
} else {
  rl.question('Enter new version (or major/minor/patch): ', (answer) => {
    if (['major', 'minor', 'patch'].includes(answer.trim())) {
      newVersion = incrementVersion(currentVersion, answer.trim());
    } else if (/^[0-9]+\.[0-9]+\.[0-9]+$/.test(answer.trim())) {
      newVersion = answer.trim();
    }
    
    if (newVersion) {
      runRelease(newVersion);
    } else {
      console.error('Invalid version or release type.');
      process.exit(1);
    }
    rl.close();
  });
}

function runRelease(version) {
  console.log(`\nReleasing version: v${version}...\n`);
  
  // 1. Update files
  for (const file of filesToUpdate) {
    updateFile(file.path, file.regex, file.replacement, version);
  }
  
  try {
    // 2. Sync Cargo.lock
    console.log('Syncing Cargo.lock...');
    execSync('cd src-tauri && cargo update -p Jedi', { stdio: 'inherit' });
    
    // 3. Git commit and tag
    console.log('Committing and tagging...');
    execSync('git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock', { stdio: 'inherit' });
    execSync(`git commit -m "chore: release v${version}"`, { stdio: 'inherit' });
    execSync(`git tag v${version}`, { stdio: 'inherit' });
    
    console.log(`\nSuccess! Version v${version} created.`);
    console.log('Run the following command to push to GitHub and trigger release workflow:');
    console.log(`\x1b[32mgit push origin main && git push origin v${version}\x1b[0m`);
    
  } catch (error) {
    console.error('\nAn error occurred during the release process:');
    console.error(error.message);
    process.exit(1);
  }
}
