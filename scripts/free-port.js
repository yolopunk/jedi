import { execFileSync } from 'child_process';

const rawPort = process.argv[2] ?? process.env.PORT ?? '31420';
const port = Number.parseInt(rawPort, 10);

if (!Number.isInteger(port) || port <= 0) {
  process.exit(1);
}

function killByPid(pid) {
  if (!pid) return;
  try {
    if (process.platform === 'win32') {
      execFileSync('taskkill', ['/F', '/PID', String(pid)], { stdio: 'ignore' });
    } else {
      process.kill(Number(pid), 'SIGKILL');
    }
  } catch (_) {}
}

function findPids() {
  try {
    if (process.platform === 'win32') {
      // plain `netstat -ano` (no `-p tcp`) is required: `-p tcp` omits
      // IPv6-only listeners like [::1]:31420, which vite binds on Windows
      const output = execFileSync('netstat', ['-ano'], { encoding: 'utf8' });
      return Array.from(
        new Set(
          output
            .split('\n')
            .map((line) => line.trim())
            .filter((line) => line.includes('LISTENING') && line.includes(`:${port}`))
            .map((line) => line.split(/\s+/).pop())
            .filter(Boolean),
        ),
      );
    }

    const output = execFileSync('lsof', ['-ti', `tcp:${port}`], { encoding: 'utf8' });
    return Array.from(new Set(output.split('\n').map((pid) => pid.trim()).filter(Boolean)));
  } catch (_) {
    return [];
  }
}

for (const pid of findPids()) {
  killByPid(pid);
}
