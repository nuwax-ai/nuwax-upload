#!/usr/bin/env node
// nuwax-upload — npm 主包入口
//
// 根据 process.platform / process.arch 定位对应平台子包里的原生二进制,
// 透传执行并转发 stdio / argv / 退出码 / 信号。零依赖。

import { spawn } from 'node:child_process';
import { createRequire } from 'node:module';
import { dirname, join } from 'node:path';

// 平台键(process.platform-process.arch)→ 子包名 映射
const PLATFORM_PKG = {
  'darwin-arm64': 'nuwax-upload-darwin-arm64',
  'darwin-x64': 'nuwax-upload-darwin-x64',
  'linux-arm64': 'nuwax-upload-linux-arm64',
  'linux-x64': 'nuwax-upload-linux-x64',
  'win32-x64': 'nuwax-upload-win32-x64',
};

const key = `${process.platform}-${process.arch}`;
const pkg = PLATFORM_PKG[key];
if (!pkg) {
  console.error(`nuwax-upload: unsupported platform/arch: ${key}`);
  process.exit(1);
}

// 通过 resolve 子包的 package.json 稳健拿到其安装目录,
// 不依赖子包的 "bin"(避免平台特定二进制被 npm 全局 link 的冲突)。
const require = createRequire(import.meta.url);
let binDir;
try {
  binDir = dirname(require.resolve(`${pkg}/package.json`));
} catch {
  console.error(`nuwax-upload: platform package '${pkg}' not installed.`);
  console.error(`Try reinstalling: npm install nuwax-upload`);
  process.exit(1);
}

const binName = process.platform === 'win32' ? 'nuwax-upload.exe' : 'nuwax-upload';
const binPath = join(binDir, binName);

const child = spawn(binPath, process.argv.slice(2), { stdio: 'inherit' });

// 转发常见终止信号给子进程,保证 Ctrl+C / kill 行为符合预期。
for (const sig of ['SIGINT', 'SIGTERM', 'SIGHUP']) {
  process.on(sig, () => child.kill(sig));
}
child.on('error', (e) => {
  console.error('nuwax-upload:', e.message);
  process.exitCode = 1;
});
child.on('exit', (code) => {
  process.exitCode = code ?? 1;
});
